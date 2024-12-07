use rand::{thread_rng, Rng};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Fragment, Packet};

pub fn new_test_fragment_packet(routing_vector: &[NodeId]) -> Packet {
    Packet::new_fragment(
        SourceRoutingHeader::with_first_hop(routing_vector.to_vec()),
        thread_rng().gen_range(0..256),
        Fragment::from_string(0, 1, String::new()),
    )
}
