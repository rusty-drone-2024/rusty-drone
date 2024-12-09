#![cfg(test)]
mod extra_flood;
mod normal_flood;

use crate::testing_utils::data::new_flood_request;
use crate::testing_utils::Network;
use std::collections::HashSet;
use std::time::Duration;
use wg_2024::network::NodeId;
use wg_2024::packet::PacketType;


/// assuming the topology as a client at 0
/// Connected with a drone 1
fn assert_topology_of_drones(
    amount: usize,
    topology: &[(NodeId, NodeId)],
    timeout: Duration,
) {
    let net = Network::create_and_run(
        amount,
        &topology,
        &[0],
    );

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    let mut hash_set = HashSet::new();

    while let Some(packet) = net.recv_as_client(0, timeout) {
        if let PacketType::FloodResponse(ref flood_res) = packet.pack_type {
            let trace = flood_res.path_trace.iter().map(|x| x.0);
            trace.clone().skip(1).zip(trace).for_each(|(a, b)|{
                if a < b {
                    hash_set.insert((a, b));
                } else {
                    hash_set.insert((b, a));
                }

            });

        } else if let PacketType::FloodRequest(_) = packet.pack_type {
        } else {
            panic!("Received {}", packet);
        }
    }

    hash_set.insert((0 as NodeId, 1 as NodeId));
    
    assert_eq!(hash_set.len(), topology.len(), "Wrong len");

    let mut result = hash_set.into_iter().collect::<Vec<_>>();
    let expected = topology.to_vec();
    let mut expected = expected.into_iter().map(|(a, b)|{
        if a < b {
            (a,b)
        } else {
            (b,a)
        }
    }).collect::<Vec<_>>();

    result.sort_by(|(a1, b1), (a2, b2)| { a1.cmp(a2).then(b1.cmp(b2)) });
    expected.sort_by(|(a1, b1), (a2, b2)| { a1.cmp(a2).then(b1.cmp(b2)) });
    assert_eq!(expected, result);
}