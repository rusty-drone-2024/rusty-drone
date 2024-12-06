#![cfg(test)]
use crate::testing_utils::test_initialization;
use crossbeam_channel::unbounded;
use wg_2024::controller::DroneCommand;
use wg_2024::packet::Packet;

#[test]
fn test_drone_new() {
    let (options, drone) = test_initialization();

    assert_eq!(drone.id, options.id);
    assert!(drone.controller_send.same_channel(&options.controller_send));
    assert!(drone.controller_recv.same_channel(&options.controller_recv));
    assert!(drone.packet_recv.same_channel(&options.packet_recv));
    assert_eq!(drone.packet_send.len(), options.packet_send.len());
    assert_eq!(drone.pdr, options.pdr);
}

// Commands
#[test]
fn test_drone_command_crash() {
    let (_options, mut drone) = test_initialization();
    
    assert!(drone.handle_commands(DroneCommand::Crash));
}

#[test]
fn test_drone_command_set_packet_drop_rate() {
    let (_options, mut drone) = test_initialization();
    
    let pdr = 0.123;
    assert!(!drone.handle_commands(DroneCommand::SetPacketDropRate(pdr)));
    assert_eq!(drone.pdr, pdr);
}

#[test]
fn test_drone_command_add_sender() {
    let (options, mut drone) = test_initialization();

    let node_id = 42;
    let (packet_send, _) = unbounded::<Packet>();

    assert!(!drone.handle_commands(DroneCommand::AddSender(node_id, packet_send.clone())));
    assert!(drone.packet_send.contains_key(&node_id));
}

#[test]
fn test_drone_command_remove_sender(){
    let (_options, mut drone) = test_initialization();
    
    let node_id = 42;
    let (packet_send, _) = unbounded::<Packet>();
    
    drone.handle_commands(DroneCommand::AddSender(node_id, packet_send.clone()));
    drone.packet_send.contains_key(&node_id);
    
    assert!(!drone.handle_commands(DroneCommand::RemoveSender(node_id)));
    assert!(!drone.packet_send.contains_key(&node_id));
}


