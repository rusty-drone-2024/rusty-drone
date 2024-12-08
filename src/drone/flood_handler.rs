use crate::drone::utils::new_flood_response;
use crate::drone::RustyDrone;
use crate::{extract, extract_mut};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{NodeType, Packet, PacketType};

impl RustyDrone {
    pub(super) fn handle_flood_request(&self, packet: &Packet, already_rec: bool) {
        if already_rec {
            let response_packet = self.respond_old_flood(packet);
            self.send_packet(&response_packet);
        } else {
            let (response_packet, previous_hop) = self.respond_new_flood(packet);
            self.flood_packet(&response_packet, previous_hop);
        }
    }

    /// need to create flood response
    pub(super) fn respond_old_flood(&self, packet: &Packet) -> Packet {
        let flood = extract!(packet.pack_type, PacketType::FloodRequest).unwrap();
        let mut flood_res = new_flood_response(flood);

        flood_res.path_trace.push((self.id, NodeType::Drone));
        let mut hops = flood_res
            .path_trace
            .iter()
            .map(|(node_id, _)| *node_id)
            .rev()
            .collect::<Vec<_>>();

        if hops.last() != Some(&flood.initiator_id) {
            hops.push(flood.initiator_id);
        }

        Packet::new_flood_response(
            SourceRoutingHeader { hop_index: 1, hops },
            packet.session_id,
            flood_res,
        )
    }

    /// need to update flood request
    pub(super) fn respond_new_flood(&self, packet: &Packet) -> (Packet, NodeId) {
        let mut packet = packet.clone();
        let flood = extract_mut!(packet.pack_type, PacketType::FloodRequest).unwrap();

        let prev_hop = flood
            .path_trace
            .last()
            .map(|x| x.0)
            .unwrap_or(flood.initiator_id);
        flood.path_trace.push((self.id, NodeType::Drone));
        (packet, prev_hop)
    }
}
