#![cfg(test)]

mod drone_command;
mod drone_packet;

use crate::testing_utils::{test_initialization_with_value};

#[test]
fn test_drone_new() {
    let pdr = 0.3;
    let id = 5;
    let (options, drone) = test_initialization_with_value(id, pdr);

    assert_eq!(drone.id, id);
    assert!(drone.controller_send.same_channel(&options.controller_send));
    assert!(drone.controller_recv.same_channel(&options.controller_recv));
    assert!(drone.packet_recv.same_channel(&options.packet_recv));
    assert_eq!(drone.packet_send.len(), options.packet_send.len());
    assert_eq!(drone.pdr, pdr);
}