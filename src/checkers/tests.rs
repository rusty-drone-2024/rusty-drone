#![cfg(test)]

use crate::testing_utils::data::new_test_fragment_packet;
use crate::testing_utils::Network;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(2000);

#[test]
fn test_drone_packet_fragment() {
    let mut net = Network::new(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);

    let mut packet = new_test_fragment_packet(&[0, 1, 2, 3, 4]);
    net.get_drone_packet_adder_channel(1)
        .try_send(packet.clone())
        .unwrap();

    net.start_async(&[1, 2, 3]);
    let response = net
        .get_drone_packet_remover_channel(4)
        .recv_timeout(TIMEOUT)
        .unwrap();

    (&mut packet.routing_header).hop_index = 4;
    assert_eq!(packet, response);
}
