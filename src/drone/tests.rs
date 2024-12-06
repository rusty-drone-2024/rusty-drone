#![cfg(test)]

use crate::drone::MyDrone;
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::borrow::Borrow;
use std::collections::HashMap;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

struct DroneCreationArgs {
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
}

impl DroneCreationArgs {
    fn new() -> Self {
        let (controller_send, _) = unbounded::<DroneEvent>();
        let (_, controller_recv) = unbounded::<DroneCommand>();
        let (_, packet_recv) = unbounded::<Packet>();
        let packet_send = HashMap::<NodeId, Sender<Packet>>::new();
        return DroneCreationArgs {
            id: 1,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr: 0.5f32,
        };
    }
}

fn create_drone(args: &DroneCreationArgs) -> MyDrone {
    return MyDrone::new(
        args.id,
        args.controller_send.clone(),
        args.controller_recv.clone(),
        args.packet_recv.clone(),
        args.packet_send.clone(),
        args.pdr,
    );
}

#[test]
fn test_drone_new() -> () {
    let args = DroneCreationArgs::new();
    let drone = create_drone(&args);

    assert_eq!(drone.id, args.id);
    assert!(drone.controller_send.same_channel(&args.controller_send));
    assert!(drone.controller_recv.same_channel(&args.controller_recv));
    assert!(drone.packet_recv.same_channel(&args.packet_recv));
    assert_eq!(drone.packet_send.len(), args.packet_send.len());
    assert_eq!(drone.pdr, args.pdr);
}

// Commands
#[test]
fn test_drone_command_crash() -> () {
    let args = DroneCreationArgs::new();
    let mut drone = create_drone(&args);
    let crashed = drone.handle_commands(DroneCommand::Crash);
    assert!(crashed);
}

#[test]
fn test_drone_command_set_packet_drop_rate() -> () {
    let args = DroneCreationArgs::new();
    let pdr = 0.123;
    assert_ne!(args.pdr, pdr);
    let mut drone = create_drone(&args);
    let crashed = drone.handle_commands(DroneCommand::SetPacketDropRate(pdr));
    if (crashed) {
        return ();
    }

    assert_eq!(drone.pdr, pdr);
}

#[test]
fn test_drone_command_remove_sender() -> () {
    let args = DroneCreationArgs::new();
    let node_id = 42;
    assert!(!args.packet_send.contains_key(&node_id));
    let mut drone = create_drone(&args);
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
    let args = DroneCreationArgs::new();
    let node_id = 42;
    assert!(!args.packet_send.contains_key(&node_id));
    let mut drone = create_drone(&args);
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
