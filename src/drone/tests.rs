#![cfg(test)]

use crate::drone::MyDrone;
use crossbeam_channel::{unbounded, Sender};
use std::borrow::Borrow;
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
        command_recv.clone(),
        packet_recv.clone(),
        neighbors_send.clone(),
        pdr,
    );

    assert_eq!(drone.id, id);
    assert!(drone.controller_send.same_channel(&event_send));
    assert!(drone.controller_recv.same_channel(&command_recv));
    assert!(drone.packet_recv.same_channel(&packet_recv));
    assert_eq!(drone.packet_send.len(), neighbors_send.len());
    assert_eq!(drone.pdr, pdr);
}
