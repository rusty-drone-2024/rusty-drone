#![cfg(test)]

use crate::checkers::TIMEOUT;
use crate::testing_utils::data::{new_flood_request, new_flood_request_with_path};
use crate::testing_utils::Network;
use wg_2024::packet::NodeType;

#[test]
fn test_easiest_flood() {
    let net = Network::create_and_run(4, &[(0, 1), (1, 2), (1, 3)], &[0, 2, 3]);

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    let expected = new_flood_request_with_path(5, 7, 0, &[(1, NodeType::Drone)]);
    assert_eq!(expected, net.recv_as_client(2, TIMEOUT).unwrap());
    assert_eq!(expected, net.recv_as_client(3, TIMEOUT).unwrap());
}