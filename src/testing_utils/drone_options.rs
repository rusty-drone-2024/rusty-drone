use std::collections::HashMap;
use crossbeam_channel::{unbounded, Receiver, Sender};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use crate::drone::MyDrone;

pub struct DroneOptions {
    pub id: u8,
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub pdr: f32,
    // resto private
    _packet_drone_in: Sender<Packet>,
    _command_send: Sender<DroneCommand>,
    _event_recv: Receiver<DroneEvent>,
}

impl DroneOptions {
    pub fn new() -> Self {
        let (controller_send, _event_recv) = unbounded::<DroneEvent>();
        let (_command_send, controller_recv) = unbounded::<DroneCommand>();
        let (_packet_drone_in, packet_recv) = unbounded::<Packet>();
        let packet_send = HashMap::<NodeId, Sender<Packet>>::new();
        Self {
            id: 1,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr: 0.5f32,
            
            _packet_drone_in,
            _command_send,
            _event_recv,
        }
    }

    pub fn create_drone(&self) -> MyDrone {
        MyDrone::new(
            self.id,
            self.controller_send.clone(),
            self.controller_recv.clone(),
            self.packet_recv.clone(),
            self.packet_send.clone(),
            self.pdr,
        )
    }
}