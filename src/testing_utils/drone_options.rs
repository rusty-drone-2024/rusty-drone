use std::collections::HashMap;
use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use crate::drone::MyDrone;

#[allow(dead_code)]
pub struct DroneOptions {
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub packet_drone_in: Sender<Packet>,
    pub command_send: Sender<DroneCommand>,
    pub event_recv: Receiver<DroneEvent>,
}

impl DroneOptions {
    pub fn new() -> Self {
        let (controller_send, event_recv) = unbounded::<DroneEvent>();
        let (command_send, controller_recv) = unbounded::<DroneCommand>();
        let (packet_drone_in, packet_recv) = unbounded::<Packet>();
        let packet_send = HashMap::<NodeId, Sender<Packet>>::new();
        Self {
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            packet_drone_in,
            command_send,
            event_recv,
        }
    }

    pub fn create_drone(&self, id: NodeId, pdr: f32) -> MyDrone {
        MyDrone::new(
            id,
            self.controller_send.clone(),
            self.controller_recv.clone(),
            self.packet_recv.clone(),
            self.packet_send.clone(),
            pdr,
        )
    }
}