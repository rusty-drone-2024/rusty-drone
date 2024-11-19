use crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use std::time::Instant;
use wg_2024::controller::Command;
use wg_2024::drone::{Drone, DroneOptions};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Nack, NackType, Packet, PacketType};

#[allow(dead_code)]
struct MyDrone {
    id: NodeId,
    sim_contr_send: Sender<Command>, // TODO missing command response
    sim_contr_recv: Receiver<Command>,
    pdr: u8,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
}

impl Drone for MyDrone {
    fn new(options: DroneOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.sim_contr_send,
            sim_contr_recv: options.sim_contr_recv,
            packet_recv: options.packet_recv,
            pdr: (options.pdr * 100.0) as u8,
            packet_send: HashMap::new(),
        }
    }

    fn run(&mut self) {
        loop {
            select! {
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res{
                        self.handle_normal_packets(packet);
                    }else {
                        // It is disconnected
                    }
                },
                recv(self.sim_contr_recv) -> command_res => {
                    if let Ok(command) = command_res{
                        self.handle_command_packets(command);
                    }else {
                        // It is disconnected
                    }
                }
            }
        }
    }
}

// Command handling part
impl MyDrone {
    fn handle_command_packets(&self, _packet: Command) {
        todo!()
    }
}

// Packet handling part
impl MyDrone {
    fn handle_normal_packets(&self, packet: Packet) {
        let routing = packet.routing_header;
        let session_id = packet.session_id;

        let res = self.forward(packet);

        if let Err(nack_type) = res {
            let node_id = self.get_hop_id(&routing, -1);

            // If it is possible to send error
            // TODO this should be discussed in WG
            if let Ok(node_id) = node_id {
                if let Some(channel) = self.packet_send.get(&node_id) {
                    // TODO add new to struct packet
                    MyDrone::send(
                        Packet {
                            pack_type: PacketType::Nack(Nack {
                                fragment_index: 0, // TODO should be optional
                                time_of_fail: Instant::now(),
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
        let next_node_id = self.get_hop_id(&packet.routing_header, 1)?;

        let channel = self
            .packet_send
            .get(&next_node_id)
            .ok_or(NackType::ErrorInRouting(next_node_id))?;

        MyDrone::send(packet, channel);
        Ok(())
    }

    fn get_hop_id(&self, routing: &SourceRoutingHeader, diff: i64) -> Result<NodeId, NackType> {
        // Nack type should be changed to the appropriate one
        // But it is still not in the PR
        let pos = routing
            .iter()
            .position(|x| x.eq(&self.id))
            .ok_or(NackType::Dropped())?;

        // Error should be DestinationIsServer
        let node_id = routing
            .get((pos as i64 + diff) as usize) // this is bad
            .ok_or(NackType::Dropped())?;

        Ok(*node_id)
    }

    fn invert_routing(&self, _routing: &SourceRoutingHeader) -> SourceRoutingHeader {
        todo!()
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
