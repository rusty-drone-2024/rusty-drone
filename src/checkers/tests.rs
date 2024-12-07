#![cfg(test)]

use crate::testing_utils::data::new_test_fragment_packet;
use crate::testing_utils::Network;
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_millis(100);

#[test]
fn test_drone_packet_3_hop() {
    let mut net = Network::new(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    net.start_async(&[1, 2, 3]);
    
    let mut packet = new_test_fragment_packet(&[0, 1, 2, 3, 4]);
    net.get_drone_packet_adder_channel(1)
        .try_send(packet.clone())
        .unwrap();
    
    let response = net
        .get_drone_packet_remover_channel(4)
        .recv_timeout(TIMEOUT)
        .unwrap();

    (&mut packet.routing_header).hop_index = 4;
    assert_eq!(packet, response);
}

#[test]
fn test_drone_packet_255_hop() {
    let mut net = Network::new(256, 
                               &(0..255).map(|i| (i,i + 1)).collect::<Vec<_>>());
    net.start_async(&(1..255).collect::<Vec<_>>());
    
    let mut packet = new_test_fragment_packet(&(0..=255).collect::<Vec<_>>());
    
    let time = Instant::now();
    net.get_drone_packet_adder_channel(1)
        .try_send(packet.clone())
        .unwrap();
    
    let response = net
        .get_drone_packet_remover_channel(255)
        .recv_timeout(TIMEOUT)
        .unwrap();
    let elapsed = time.elapsed();

    (&mut packet.routing_header).hop_index = 255;
    assert_eq!(packet, response);
    assert!(elapsed.le(&Duration::from_millis(40)));
}
