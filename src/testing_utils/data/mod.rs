use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Fragment, Nack, NackType, Packet};

pub fn new_test_fragment_packet(routing_vector: &[NodeId], session_id: u64) -> Packet {
    Packet::new_fragment(
        SourceRoutingHeader::with_first_hop(routing_vector.to_vec()),
        session_id,
        Fragment::from_string(0, 1, String::new()),
    )
}

pub fn new_test_nack(
    hops: &[NodeId],
    nack_type: NackType,
    session_id: u64,
    hop_index: usize,
) -> Packet {
    Packet::new_nack(
        SourceRoutingHeader::new(hops.to_vec(), hop_index),
        session_id,
        Nack {
            fragment_index: 0,
            nack_type,
        },
    )
}

pub fn new_forwarded(packet: &Packet) -> Packet {
    let mut packet = packet.clone();
    packet.routing_header.increase_hop_index();
    packet
}