#![cfg(test)]
use crate::drone::RustyDrone;
use crate::testing_utils::DroneOptions;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread;
use std::time::Duration;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

pub struct Network {
    sc_event_rcv: Receiver<DroneEvent>,
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
        let (sc_event_send, sc_event_rcv) = unbounded::<DroneEvent>();
        let mut options = (0..amount)
            .map(|_| DroneOptions::new_with_sc(sc_event_send.clone(), sc_event_rcv.clone()))
            .collect::<Vec<_>>();

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
                let drone = if client.contains(&(i as NodeId)) {
                    None
                } else {
                    Some(options.create_drone(i as NodeId, 0.0))
                };

                NetworkDrone { options, drone }
            })
            .collect();

        Self {
            nodes,
            sc_event_rcv,
        }
    }

    pub fn add_connections(&mut self, start: NodeId, end: NodeId) {
        let options_start = &self.nodes[start as usize].options;
        let options_end = &self.nodes[end as usize].options;

        self.send_as_simulation_controller_to(
            start,
            DroneCommand::AddSender(end, options_end.packet_drone_in.clone()),
        );

        self.send_as_simulation_controller_to(
            end,
            DroneCommand::AddSender(start, options_start.packet_drone_in.clone()),
        );
    }

    fn get_drone_packet_adder_channel(&self, node_id: NodeId) -> Sender<Packet> {
        let options = &self.nodes[node_id as usize].options;
        options.packet_drone_in.clone()
    }

    fn get_drone_packet_remover_channel(&self, node_id: NodeId) -> Receiver<Packet> {
        let options = &self.nodes[node_id as usize].options;
        options.packet_recv.clone()
    }

    pub fn simulation_controller_event_receiver(&self) -> Receiver<DroneEvent> {
        let options = &self.nodes.first().unwrap().options;
        options.event_recv.clone()
    }

    pub fn send_as_simulation_controller_to(&self, node_id: NodeId, command: DroneCommand) {
        self.nodes[node_id as usize]
            .options
            .command_send
            .send(command)
            .unwrap()
    }

    pub fn send_as_client(&self, node_id: NodeId, packet: Packet) -> Option<()> {
        let to = packet.routing_header.current_hop();
        if let Some(to) = to {
            return self.send_to_dest_as_client(node_id, to, packet);
        }
        None
    }

    pub(crate) fn send_to_dest_as_client(
        &self,
        from: NodeId,
        to: NodeId,
        packet: Packet,
    ) -> Option<()> {
        let neighbour = self.nodes[from as usize].options.packet_send.get(&to);
        if let Some(neighbour) = neighbour {
            return neighbour.send(packet).ok();
        }

        None
    }

    pub fn recv_as_client(&self, node_id: NodeId, timeout: Duration) -> Option<Packet> {
        self.get_drone_packet_remover_channel(node_id)
            .recv_timeout(timeout)
            .ok()
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

    /// Start some drone as fake client
    /// They only respond to FloodRequest
    fn start_fake_clients_async(&mut self, fake_clients: &[NodeId]) {
        todo!("{:?}", fake_clients)
    }
}
