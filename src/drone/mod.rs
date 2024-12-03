#![allow(unused)]

mod tests;
use crossbeam_channel::{select_biased, unbounded, Receiver, Sender};
use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;
use wg_2024::config::Config;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Nack, NackType, Packet, PacketType};

#[allow(dead_code)]
pub struct MyDrone {
    id: NodeId,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
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
            id: id,
            controller_send: controller_send,
            controller_recv: controller_recv,
            packet_recv: packet_recv,
            pdr: pdr,
            packet_send: HashMap::new(),
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
    fn handle_normal_packets(&mut self, packet: Packet) {
        let routing = packet.routing_header.clone();
        let session_id = packet.session_id;

        //Forward the packet
        let res = self.forward(packet);

        if let Err(nack_type) = res {
            let node_id = self.get_hop_id(&routing, -1);

            if let Ok(node_id) = node_id {
                if let Some(channel) = self.packet_send.get(&node_id) {
                    MyDrone::send(
                        Packet {
                            pack_type: PacketType::Nack(Nack {
                                fragment_index: 0,
                                nack_type,
                            }),
                            routing_header: self.invert_routing(&routing),
                            session_id,
                        },
                        channel,
                    );
                }
            }
        }
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

    fn invert_routing(&self, routing: &SourceRoutingHeader) -> SourceRoutingHeader {
        let mut new_routing = SourceRoutingHeader {
            hop_index: 0,
            hops: Vec::new(),
        };
        for node in routing.hops.iter().rev() {
            new_routing.hops.push(*node);
            if self.id == *node {
                break;
            }
        }
        new_routing
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
