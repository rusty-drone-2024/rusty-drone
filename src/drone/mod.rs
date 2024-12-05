#![allow(unused)]

mod tests;
use crossbeam_channel::{select_biased, unbounded, Receiver, RecvError, Sender};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::time::Instant;
use wg_2024::config::Config;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::NackType::{DestinationIsDrone, ErrorInRouting, UnexpectedRecipient};
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
                        self.handle_packet(packet)
                    }
                },
            }
        }

        // crashing
        while let Ok(packet) = self.packet_recv.recv() {
            self.handle_packet(packet);
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
    fn handle_packet(&mut self, mut packet: Packet) {
        // Do custom handling for floods
        if let PacketType::FloodRequest(_) = packet.pack_type {
            let response_packet = self.handle_flood_request(packet);

            // need to split between request and response
        } else {
            let res = self.handle_normal_types(packet);
            if let Some(response_packet) = res {
                self.send_packet(response_packet);
            }
        }
    }
}

impl MyDrone {
    /// Return wheter it should crash or not
    fn handle_normal_types(&self, mut packet: Packet) -> Option<Packet> {
        let droppable = matches!(packet.pack_type, PacketType::MsgFragment(_));
        let routing = &mut packet.routing_header;

        if routing.current_hop() != Some(self.id) {
            if !droppable {
                // the protocol say so but it is just dumb
                self.use_shortcut(packet);
                return None;
            }
            return Some(create_nack(packet, UnexpectedRecipient(self.id)));
        }

        //TODO may be broken check all function in repo
        if routing.is_last_hop() {
            if !droppable {
                // cannot nack only fragment, rest will be dropped
                return None;
            }
            return Some(create_nack(packet, DestinationIsDrone));
        }

        // cannot be done before as asked by the protocol
        routing.increase_hop_index();

        // next hop must exist
        let next_hop = routing.next_hop()?;
        if !self.packet_send.contains_key(&next_hop) {
            if !droppable {
                self.use_shortcut(packet);
                return None;
            }

            return Some(create_nack(packet, NackType::ErrorInRouting(next_hop)));
        }

        if droppable && self.should_drop() {
            return Some(create_nack(packet, NackType::Dropped));
        }

        // forward if all succeed
        Some(packet)
    }

    /// return the response to be sent
    fn handle_flood_request(&self, mut packet: Packet) -> Packet {
        //TODO
        todo!()
    }
}

// Packet handling part
impl MyDrone {
    fn should_drop(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0) < self.pdr
    }
}

// Common part
impl MyDrone {
    fn send<T>(to_send: T, channel: &Sender<T>) {
        channel.send(to_send);
    }

    fn use_shortcut(&self, packet: Packet) {
        self.controller_send
            .send(DroneEvent::ControllerShortcut(packet));
    }

    fn send_packet(&self, packet: Packet) {
        //TODO
    }
}

fn create_nack(p0: Packet, p1: NackType) -> Packet {
    // TODO invert srh and set index to 0 (or 1 cannot remeber)
    // TODO create a new packet of type nack
    todo!()
}
