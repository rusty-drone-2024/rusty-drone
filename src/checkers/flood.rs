use crate::checkers::TIMEOUT;
use crate::testing_utils::data::{new_flood_request, new_flood_request_with_path};
use crate::testing_utils::Network;
use std::collections::HashMap;
use std::time::Duration;
use wg_2024::network::NodeId;
use wg_2024::packet::{NodeType, PacketType};

fn assert_topology_on_client(
    net: Network,
    mut expected: Vec<(NodeId, NodeType)>,
    timeout: Duration,
) {
    let mut hash_map = HashMap::new();

    while let Some(packet) = net.recv_as_client(2, timeout) {
        if let PacketType::FloodResponse(flood_res) = packet.pack_type {
            for (node_id, node_type) in flood_res.path_trace {
                if let Some(old_type) = hash_map.get(&node_id) {
                    assert_eq!(*old_type, node_type);
                } else {
                    hash_map.insert(node_id, node_type);
                }
            }
        } else if let PacketType::FloodRequest(_) = packet.pack_type {
        } else {
            panic!("Received {}", packet);
        }
    }

    //assert_eq!(req_n, 2, "Wrong number of request");
    assert_eq!(hash_map.len(), expected.len(), "Wrong len");

    let mut result = hash_map.into_iter().collect::<Vec<_>>();

    result.sort_by(|(id1, _), (id2, _)| id1.cmp(id2));
    expected.sort_by(|(id1, _), (id2, _)| id1.cmp(id2));
    assert_eq!(result, expected);
}

#[test]
fn test_easiest_flood() {
    let net = Network::create_and_run(4, &[(0, 1), (1, 2), (1, 3)], &[0, 2, 3]);

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    let expected = new_flood_request_with_path(5, 7, 0, &[(1, NodeType::Drone)]);
    assert_eq!(expected, net.recv_as_client(2, TIMEOUT).unwrap());
    assert_eq!(expected, net.recv_as_client(3, TIMEOUT).unwrap());
}

#[test]
fn test_loop_flood() {
    let net = Network::create_and_run(4, &[(0, 1), (1, 2), (1, 3), (2, 3)], &[0]);

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    assert_topology_on_client(
        net,
        vec![
            (1, NodeType::Drone),
            (2, NodeType::Drone),
            (3, NodeType::Drone),
        ],
        Duration::from_secs(1),
    );
}


#[test]
fn test_hard_loop_flood() {
    let net = Network::create_and_run(6, &[(0, 1), (2, 1), (3,1), (3,2), (4,1), (4,2), (4,3), (5, 1), (5, 2), (5, 3), (5, 4)], &[0]);

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    assert_topology_on_client(
        net,
        vec![
            (1, NodeType::Drone),
            (2, NodeType::Drone),
            (3, NodeType::Drone),
            (4, NodeType::Drone),
            (5, NodeType::Drone)
        ],
        Duration::from_secs(10),
    );
}

