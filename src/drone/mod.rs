#![allow(unused)]

mod tests;
use crossbeam_channel::{select_biased, unbounded, Receiver, Sender};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use wg_2024::config::Config;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::NackType::{DestinationIsDrone, UnexpectedRecipient};
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
        loop {
            select_biased! {
                recv(self.controller_recv) -> command => {
                    if let Ok(command) = command {
                        if let DroneCommand::Crash = command {
                            println!("drone {} crashed", self.id);
                            break;
                        }
                        self.handle_command_packets(command);
                    }
                }
                recv(self.packet_recv) -> packet => {
                    if let Ok(packet) = packet {
                        self.handle_normal_packets(packet);
                    }
                },
            }
        }
    }
}

// Command handling part
impl MyDrone {
    fn handle_command_packets(&mut self, command: DroneCommand) {
        match command {
            DroneCommand::RemoveSender(node_id) => {
                self.packet_send.remove(&node_id);
            }
            DroneCommand::AddSender(node_id, sender) => {
                self.packet_send.insert(node_id, sender);
            }
            DroneCommand::SetPacketDropRate(pdr) => self.pdr = pdr,
            DroneCommand::Crash => todo!(),
        }
    }
}

// Packet handling part
impl MyDrone {
    fn handle_normal_packets(&mut self, mut packet: Packet) {
        // destructure packet
        let mut routing_header = packet.routing_header;
        let session_id = packet.session_id;
        let pack_type = packet.pack_type;

        // protocol step 1:
        if routing_header.valid_hop_index() && routing_header.current_hop().unwrap() == self.id {
            // protocol step 2:
            routing_header.increase_hop_index();

            // protocol step 3:
            if !routing_header.is_last_hop() {
            } else {
                match pack_type {
                    PacketType::MsgFragment(fragment) => self.handle_nack(
                        routing_header,
                        session_id,
                        fragment.fragment_index,
                        DestinationIsDrone,
                    ),
                    _ => self.handle_nack(routing_header, session_id, 0, DestinationIsDrone),
                }
            }
        } else {
            match pack_type {
                PacketType::MsgFragment(fragment) => self.handle_nack(
                    routing_header,
                    session_id,
                    fragment.fragment_index,
                    UnexpectedRecipient(self.id),
                ),
                _ => self.handle_nack(routing_header, session_id, 0, UnexpectedRecipient(self.id)),
            }
        }
    }

    fn handle_fragment(
        &mut self,
        routing_header: SourceRoutingHeader,
        session_id: u64,
        fragment: Fragment,
    ) {
    }
    fn handle_ack(
        &mut self,
        routing_header: SourceRoutingHeader,
        session_id: u64,
        fragment_index: u64,
    ) {
        // reverse srh
        // create ack according to protocol
        // create packet
        // send packet
    }
    fn handle_nack(
        &mut self,
        routing_header: SourceRoutingHeader,
        session_id: u64,
        fragment_index: u64,
        nack_type: NackType,
    ) {
        // reverse srh
        // create nack
        // create packet
        // send packet
    }
    fn handle_flood_request(
        &mut self,
        routing_header: SourceRoutingHeader,
        session_id: u64,
        flood_request: FloodRequest,
    ) {
    }
    fn handle_flood_response(
        &mut self,
        routing_header: SourceRoutingHeader,
        session_id: u64,
        flood_response: FloodResponse,
    ) {
    }

    fn forward(&self, packet: Packet) -> Result<(), NackType> {
        if self.should_drop() {
            return Err(NackType::Dropped);
        }
        let next_node_id = self.get_hop_id(&packet.routing_header, 1)?;

        let channel = self
            .packet_send
            .get(&next_node_id)
            .ok_or(NackType::ErrorInRouting(next_node_id))?;

        MyDrone::send(packet, channel);
        Ok(())
    }

    fn get_hop_id(&self, routing: &SourceRoutingHeader, diff: i64) -> Result<NodeId, NackType> {
        let pos = routing.hops.iter().position(|x| x.eq(&self.id)).ok_or(
            NackType::UnexpectedRecipient(routing.hops[routing.hop_index - 1]),
        )?;

        let node_id = routing
            .hops
            .get((pos as i64 + diff) as usize) // this is bad
            .ok_or(NackType::DestinationIsDrone)?;

        Ok(*node_id)
    }

    fn should_drop(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0) < self.pdr
    }
}

// Common part
impl MyDrone {
    fn send<T>(to_send: T, channel: &Sender<T>) {
        if channel.send(to_send).is_err() {
            // Boh. Do some logging?
        }
    }
}
