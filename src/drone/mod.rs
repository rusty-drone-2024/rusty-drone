mod channels;
mod command_handler;
mod flood_handler;
mod packet_handler;
mod test;
mod utils;

use crossbeam_channel::{select_biased, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

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
                    crashing = self.handle_commands(res.as_ref().unwrap());
                },
                recv(self.packet_recv) -> res => {
                    self.handle_packet(res.as_ref().unwrap(), false);
                },
            }
        }

        // crashing
        while let Ok(ref mut packet) = self.packet_recv.recv() {
            self.handle_packet(packet, true);
        }
    }
}

impl RustyDrone{
    /// return the response to be sent
    pub fn handle_packet(&mut self, packet: &Packet, crashing: bool) {
        // Do custom handling for floods
        if let PacketType::FloodRequest(ref flood) = packet.pack_type {
            if !crashing {
                self.handle_flood_request(packet.session_id, flood);
            }
        } else {
            let res = self.respond_normal_types(packet, crashing);
            if let Some(ref response_packet) = res {
                self.send_packet(response_packet);
            }
        }
    }
}
