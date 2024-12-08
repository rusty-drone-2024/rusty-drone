mod test_easiest_flood;
mod test_loop_flood;
mod test_hard_loop_flood;

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