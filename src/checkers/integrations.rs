#![cfg(test)]
use crate::testing_utils::data::new_test_fragment_packet;
use crate::testing_utils::Network;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_millis(100);

#[test]
fn test_drone_packet_1_hop() {
    let net = Network::create_and_run(3, &[(0, 1), (1, 2)], &[0, 2]);

    let mut packet = new_test_fragment_packet(&[0, 1, 2]);
    net.send_as_client(0, packet.clone());

    let response = net.recv_as_client(2, TIMEOUT).unwrap();

    (&mut packet.routing_header).hop_index = 2;
    assert_eq!(packet, response);
}

#[test]
fn test_drone_packet_3_hop() {
    let net = Network::create_and_run(5, &[(0, 1), (1, 2), (2, 3), (3, 4)], &[0, 4]);

    let mut packet = new_test_fragment_packet(&[0, 1, 2, 3, 4]);
    net.send_as_client(0, packet.clone());

    let response = net.recv_as_client(4, TIMEOUT).unwrap();

    (&mut packet.routing_header).hop_index = 4;
    assert_eq!(packet, response);
}

#[test]
fn test_drone_packet_255_hop() {
    let net = Network::create_and_run(
        256,
        &(0..255).map(|i| (i, i + 1)).collect::<Vec<_>>(),
        &[0, 255],
    );

    let mut packet = new_test_fragment_packet(&(0..=255).collect::<Vec<_>>());

    let time = Instant::now();
    net.send_as_client(0, packet.clone());

    let response = net.recv_as_client(255, TIMEOUT).unwrap();
    let elapsed = time.elapsed();

    (&mut packet.routing_header).hop_index = 255;
    assert_eq!(packet, response);
    // TODO this test can fail on slower machines
    assert!(elapsed.le(&Duration::from_millis(50)), "Too Slow");
}
