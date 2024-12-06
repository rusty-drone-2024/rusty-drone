mod test;
mod utils;

use crossbeam_channel::{select_biased, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::NackType::{DestinationIsDrone, Dropped, ErrorInRouting, UnexpectedRecipient};
use wg_2024::packet::{FloodResponse, Nack, NackType, NodeType, Packet, PacketType};

#[allow(dead_code)]
pub struct RustyDrone {
    id: NodeId,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
    received_floods: HashSet<(u64, NodeId)>,
}

impl Drone for RustyDrone {
    fn new(
        id: NodeId,
        controller_send: Sender<DroneEvent>,
        controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        pdr: f32,
    ) -> Self {
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            pdr,
            packet_send,
            received_floods: HashSet::new(),
        }
    }

    fn run(&mut self) {
        let mut crashing = false;
        while !crashing {
            select_biased! {
                recv(self.controller_recv) -> res => {
                    if let Ok(command) = res{
                        crashing = self.handle_commands(command)
                    }
                },
                recv(self.packet_recv) -> res => {
                    if let Ok(packet) = res{
                        self.handle_packet(packet, false)
                    }
                },
            }
        }

        // crashing
        while let Ok(packet) = self.packet_recv.recv() {
            self.handle_packet(packet, true);
        }
    }
}

// Command/packets handling part
impl RustyDrone {
    fn handle_commands(&mut self, command: DroneCommand) -> bool {
        match command {
            DroneCommand::Crash => return true,
            DroneCommand::SetPacketDropRate(pdr) => self.pdr = pdr,
            DroneCommand::RemoveSender(ref node_id) => {
                self.packet_send.remove(node_id);
            }
            DroneCommand::AddSender(node_id, sender) => {
                self.packet_send.insert(node_id, sender);
            }
        }

        false
    }

    /// return the response to be sent
    fn handle_packet(&mut self, packet: Packet, crashing: bool) {
        // Do custom handling for floods
        if let PacketType::FloodRequest(ref flood) = packet.pack_type {
            if !crashing {
                let already_rec = self.already_received_flood(
                    flood.flood_id,
                    flood.initiator_id,
                    packet.session_id,
                );
                self.handle_flood_request(packet, already_rec);
            }
        } else {
            let res = self.respond_normal_types(packet, crashing);
            if let Some(response_packet) = res {
                self.send_packet(response_packet);
            }
        }
    }

    fn handle_flood_request(&self, packet: Packet, already_rec: bool) {
        if already_rec || self.packet_send.len() <= 1 {
            if let Some(response_packet) = self.respond_flood_old(packet) {
                self.send_packet(response_packet);
            }
        } else {
            // Technically it is possible to receive a packet with wrong data and not
            // knowing to who not send
            if let Some((response_packet, previous_hop)) = self.respond_flood_new(packet) {
                self.flood_packet(response_packet, previous_hop);
            }
        }
    }
}

/// Respond methods
impl RustyDrone {
    /// Return wheter it should crash or not
    fn respond_normal_types(&self, mut packet: Packet, crashing: bool) -> Option<Packet> {
        let droppable = matches!(packet.pack_type, PacketType::MsgFragment(_));
        let routing = &mut packet.routing_header;

        // If unexpected packets
        if routing.current_hop() != Some(self.id) {
            // the protocol say so but it is just dumb
            return self.create_nack(packet, UnexpectedRecipient(self.id), droppable, true);
        }

        if routing.is_last_hop() {
            // cannot nack only fragment, rest will be dropped
            return self.create_nack(packet, DestinationIsDrone, droppable, false);
        }

        // next hop must exist
        let next_hop = routing.next_hop()?;
        if !self.packet_send.contains_key(&next_hop) {
            return self.create_nack(packet, ErrorInRouting(next_hop), droppable, true);
        }

        if droppable && utils::should_drop(self.pdr) {
            let _ = self
                .controller_send
                .send(DroneEvent::PacketDropped(packet.clone()));
            return self.create_nack(packet, Dropped, droppable, false);
        }

        if crashing && droppable {
            let current_hop = routing.current_hop()?;
            return self.create_nack(packet, ErrorInRouting(current_hop), droppable, false);
        }

        // forward
        // cannot be done before as asked by the protocol (should be before .is_last_hop)
        routing.increase_hop_index();
        Some(packet)
    }

    /// need to create flood response
    fn respond_flood_old(&self, packet: Packet) -> Option<Packet> {
        let mut flood;
        if let PacketType::FloodRequest(ref flood_ref) = packet.pack_type {
            flood = flood_ref.clone();
        } else {
            // Should not happen (i know it is SHIT)
            return None;
        }

        flood.path_trace.push((self.id, NodeType::Drone));
        let hops = flood
            .path_trace
            .iter()
            .map(|(node_id, _)| *node_id)
            .rev()
            .collect::<Vec<_>>();

        Some(Packet::new_flood_response(
            SourceRoutingHeader { hop_index: 1, hops },
            packet.session_id,
            FloodResponse {
                flood_id: flood.flood_id,
                path_trace: flood.path_trace,
            },
        ))
    }

    /// need to update flood request
    fn respond_flood_new(&self, mut packet: Packet) -> Option<(Packet, NodeId)> {
        let flood;
        if let PacketType::FloodRequest(ref mut flood_ref) = packet.pack_type {
            flood = flood_ref;
        } else {
            // Should not happen (i know it is SHIT)
            return None;
        }

        let prev_hop = flood.path_trace.last()?.0;
        flood.path_trace.push((self.id, NodeType::Drone));
        Some((packet, prev_hop))
    }
}

/// Utils of drone
impl RustyDrone {
    fn create_nack(
        &self,
        packet: Packet,
        nack_type: NackType,
        droppable: bool,
        is_shortcuttable: bool,
    ) -> Option<Packet> {
        if !droppable {
            if is_shortcuttable {
                self.use_shortcut(packet);
            }
            return None;
        }

        let mut reversed_routes = SourceRoutingHeader {
            hop_index: 1,
            hops: packet.routing_header.hops[0..=packet.routing_header.hop_index].to_vec(),
        };
        reversed_routes.reverse();

        Some(Packet::new_nack(
            reversed_routes,
            packet.session_id,
            Nack {
                nack_type,
                fragment_index: utils::get_fragment_index(packet.pack_type),
            },
        ))
    }

    fn already_received_flood(
        &self,
        flood_id: u64,
        initiator_id: NodeId,
        _session_id: u64,
    ) -> bool {
        // Should keep in mind all of them but will only use flood_id as per protol
        // this is broken and wont work
        // so we will see what to do
        // TODO talk with WG
        self.received_floods.contains(&(flood_id, initiator_id))
    }
}

/// Packet sending
impl RustyDrone {
    fn use_shortcut(&self, packet: Packet) {
        let _ = self
            .controller_send
            .send(DroneEvent::ControllerShortcut(packet));
    }

    fn send_packet(&self, packet: Packet) {
        let next = packet.routing_header.current_hop();
        if let Some(next_hop) = next {
            if let Some(channel) = self.packet_send.get(&next_hop) {
                let _ = channel.send(packet.clone());
                let _ = self.controller_send.send(DroneEvent::PacketSent(packet));
            }
        }
        // Ignore broken send (it is an internal problem)
    }

    fn flood_packet(&self, packet: Packet, previous_hop: NodeId) {
        for (node_id, channel) in self.packet_send.iter() {
            if *node_id != previous_hop {
                let _ = channel.send(packet.clone());
            }
        }
    }
}
