#![cfg(test)]

use crate::drone::MyDrone;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::borrow::Borrow;
use std::collections::HashMap;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

fn create_drone_with_channels() -> (
    MyDrone,
    Receiver<DroneEvent>,
    Sender<DroneCommand>,
    Sender<Packet>,
) {
    let id = 1;
    let (event_send, event_recv) = unbounded::<DroneEvent>();
    let (command_send, command_recv) = unbounded::<DroneCommand>();
    let (packet_send, packet_recv) = unbounded::<Packet>();
    let neighbors_send = HashMap::<NodeId, Sender<Packet>>::new();
    let pdr = 0.5f32;

    return (
        MyDrone::new(
            id,
            event_send.clone(),
            command_recv.clone(),
            packet_recv.clone(),
            neighbors_send.clone(),
            pdr,
        ),
        event_recv,
        command_send,
        packet_send,
    );
}

fn create_drone() -> MyDrone {
    let (drone, _, _, _) = create_drone_with_channels();
    return drone;
}

#[test]
fn test_drone_new() -> () {
    let id = 1;
    let (event_send, _) = unbounded::<DroneEvent>();
    let (_, command_recv) = unbounded::<DroneCommand>();
    let (_, packet_recv) = unbounded::<Packet>();
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

// Commands
#[test]
fn test_drone_command_crash() -> () {
    let mut drone = create_drone();
    let crashed = drone.handle_commands(DroneCommand::Crash);
    assert!(crashed);
}

#[test]
fn test_drone_command_set_packet_drop_rate() -> () {
    let pdr = 0.123;
    let mut drone = create_drone();
    let crashed = drone.handle_commands(DroneCommand::SetPacketDropRate(pdr));
    if (crashed) {
        return ();
    }

    assert_eq!(drone.pdr, pdr);
}

#[test]
fn test_drone_command_remove_sender() -> () {
    let node_id = 42;
    let mut drone = create_drone();
    let (packet_send, _) = unbounded::<Packet>();
    let mut crashed = drone.handle_commands(DroneCommand::AddSender(node_id, packet_send.clone()));
    if (crashed) {
        return ();
    }

    crashed = drone.handle_commands(DroneCommand::RemoveSender(node_id));
    if (crashed) {
        return ();
    }

    assert!(!drone.packet_send.contains_key(&node_id));
}

#[test]
fn test_drone_command_add_sender() -> () {
    let node_id = 42;
    let mut drone = create_drone();
    let (packet_send, _) = unbounded::<Packet>();
    let crashed = drone.handle_commands(DroneCommand::AddSender(node_id, packet_send.clone()));
    if (crashed) {
        return ();
    }

    match drone.packet_send.get(&node_id) {
        Some(sender) => assert!(sender.same_channel(&packet_send)),
        None => assert!(false),
    }
}
