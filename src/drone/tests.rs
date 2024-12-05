#![cfg(test)]

use crate::drone::MyDrone;
use crossbeam_channel::{unbounded, Sender};
use std::collections::HashMap;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

#[test]
fn test_drone_new() -> () {
    let id = 1;
    let (event_send, event_recv) = unbounded::<DroneEvent>();
    let (command_send, command_recv) = unbounded::<DroneCommand>();
    let (packet_send, packet_recv) = unbounded::<Packet>();
    let neighbors_send = HashMap::<NodeId, Sender<Packet>>::new();
    let pdr = 0.5f32;

    let drone = MyDrone::new(
        id,
        event_send.clone(),
        command_recv,
        packet_recv,
        neighbors_send,
        pdr,
    );
}
