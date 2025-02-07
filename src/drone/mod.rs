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
    /// Send information to the Simulation Controller.
    controller_send: Sender<DroneEvent>,
    /// Receive commands from the Simulation Controller.
    controller_recv: Receiver<DroneCommand>,
    // Channel to receive packets from our connected neighbors.
    packet_recv: Receiver<Packet>,
    /// Per (connected neighbor) NodeId, what channel to use to send packets to it.
    packet_send: HashMap<NodeId, Sender<Packet>>,
    /// Packet Drop Rate.
    pdr: f32,
    /// Store all flood requests that have been received at least once.
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
            received_floods: HashSet::new(), // Initially no received flood requests
        }
    }

    /// Continuously process messages (blocking) until we crash.
    fn run(&mut self) {
        let mut crashing = false;
        while !crashing {
            // Repeatedly try to read a message from either the Simulation Controller (priority) or one of our neighbor nodes
            select_biased! {
                recv(self.controller_recv) -> res => {
                    if let Ok(ref packet) = res{
                        crashing = self.handle_commands(packet);
                    }
                },
                recv(self.packet_recv) -> res => {
                    if let Ok(ref packet) = res{
                        self.handle_packet(packet, false);
                    }
                },
            }
        }

        // Handle remaining queued packets as crashed drone
        while let Ok(ref packet) = self.packet_recv.recv() {
            self.handle_packet(packet, true);
        }
    }
}

impl RustyDrone {
    /// Forward the packet to the respective handler function.
    fn handle_packet(&mut self, packet: &Packet, crashing: bool) {
        if let PacketType::FloodRequest(ref flood) = packet.pack_type {
            if !crashing {
                self.respond_flood_request(packet.session_id, flood);
            }
        } else {
            self.respond_normal(packet, crashing);
        }
    }
}
