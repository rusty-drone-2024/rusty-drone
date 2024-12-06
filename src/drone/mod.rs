#![allow(unused)]

mod tests;
mod utils;

use crate::drone::utils::get_fragment_index;
use crossbeam_channel::{select_biased, unbounded, Receiver, RecvError, Sender};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::time::Instant;
use wg_2024::config::Config;
use wg_2024::controller::DroneEvent::PacketDropped;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::NackType::{DestinationIsDrone, Dropped, ErrorInRouting, UnexpectedRecipient};
use wg_2024::packet::{
    Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, Packet, PacketType,
};

#[allow(dead_code)]
pub struct MyDrone {
    id: NodeId,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
    received_floods: HashSet<(u64, NodeId)>,
}

impl Drone for MyDrone {
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
impl MyDrone {
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
    fn handle_packet(&mut self, mut packet: Packet, crashing: bool) {
        // Do custom handling for floods
        if let PacketType::FloodRequest(_) = packet.pack_type {
            if !crashing {
                self.handle_flood_request(packet);
            }
        } else {
            let res = self.respond_normal_types(packet, true);
            if let Some(response_packet) = res {
                self.send_packet(response_packet);
            }
        }
    }
}

impl MyDrone {
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

    /// return the response to be sent
    fn handle_flood_request(&self, mut packet: Packet) {
        todo!() //TODO all
                // TODO sending them
    }
}

impl MyDrone {
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

    fn use_shortcut(&self, packet: Packet) {
        self.controller_send
            .send(DroneEvent::ControllerShortcut(packet));
    }

    fn send_packet(&self, packet: Packet) {
        let next = packet.routing_header.current_hop();
        if let Some(next_hop) = next {
            if let Some(channel) = self.packet_send.get(&next_hop) {
                channel.send(packet);
            }
        }
        // Ignore broken send (it is an internal problem
    }
}
