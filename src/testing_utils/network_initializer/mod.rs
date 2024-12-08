#![allow(dead_code)]

use crate::drone::RustyDrone;
use crate::testing_utils::DroneOptions;
use crossbeam_channel::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

pub struct Network {
    nodes: Vec<NetworkDrone>,
}

pub struct NetworkDrone {
    options: DroneOptions,
    drone: Option<RustyDrone>,
}

impl Network {
    pub fn create_and_run(
        amount: usize,
        connections: &[(NodeId, NodeId)],
        client: &[NodeId],
    ) -> Self {
        let mut res = Network::new(amount, connections, client);
        res.start_multi_async();
        res
    }

    /// Create vector of drone with ID from 0 to amount
    /// With the given connections
    /// Duplicated connection are ignored and the graph is not directional
    fn new(amount: usize, connections: &[(NodeId, NodeId)], client: &[NodeId]) -> Self {
        let mut options = (0..amount).map(|_| DroneOptions::new()).collect::<Vec<_>>();

        for (start, end) in connections {
            let start_input = options[*start as usize].packet_drone_in.clone();
            let end_input = options[*end as usize].packet_drone_in.clone();

            options[*start as usize].packet_send.insert(*end, end_input);
            options[*end as usize]
                .packet_send
                .insert(*start, start_input);
        }

        let nodes = options
            .into_iter()
            .enumerate()
            .map(|(i, options)| {
                if client.contains(&(i as NodeId)) {
                    return NetworkDrone {
                        options,
                        drone: None,
                    };
                }

                let drone = options.create_drone(i as NodeId, 0.0);
                NetworkDrone {
                    options,
                    drone: Some(drone),
                }
            })
            .collect();

        Self { nodes }
    }

    pub fn add_connections(&mut self, connections: &[(NodeId, NodeId)]) {
        for (start, end) in connections {
            let options_start = &self.nodes[*start as usize].options;
            let options_end = &self.nodes[*end as usize].options;

            let _ = options_start.command_send.send(DroneCommand::AddSender(
                *end,
                options_end.packet_drone_in.clone(),
            ));
            let _ = options_end.command_send.send(DroneCommand::AddSender(
                *start,
                options_start.packet_drone_in.clone(),
            ));
        }
    }

    fn get_drone_packet_adder_channel(&self, node_id: NodeId) -> Sender<Packet> {
        let options = &self.nodes[node_id as usize].options;
        options.packet_drone_in.clone()
    }

    fn get_drone_packet_remover_channel(&self, node_id: NodeId) -> Receiver<Packet> {
        let options = &self.nodes[node_id as usize].options;
        options.packet_recv.clone()
    }

    fn get_drone_command_channel(&self, node_id: NodeId) -> Sender<DroneCommand> {
        let options = &self.nodes[node_id as usize].options;
        options.command_send.clone()
    }

    fn get_drone_event_channel(&self, node_id: NodeId) -> Receiver<DroneEvent> {
        let options = &self.nodes[node_id as usize].options;
        options.event_recv.clone()
    }

    pub fn send_as_client(&self, node_id: NodeId, packet: Packet) {
        let current = packet.routing_header.current_hop();
        if let Some(current) = current {
            let neighbour = self.nodes[node_id as usize]
                .options
                .packet_send
                .get(&current);
            if let Some(neighbour) = neighbour {
                let _ = neighbour.send(packet);
            }
        }
    }

    pub fn recv_as_client(&self, node_id: NodeId, timeout: Duration) -> Option<Packet> {
        return self
            .get_drone_packet_remover_channel(node_id)
            .recv_timeout(timeout)
            .ok();
    }

    /// Start some drone
    /// Not started can be used as client and server by test
    fn start_multi_async(&mut self) {
        for node in self.nodes.iter_mut() {
            if let Some(mut drone) = node.drone.take() {
                thread::spawn(move || {
                    drone.run();
                });
            }
        }
    }
}
