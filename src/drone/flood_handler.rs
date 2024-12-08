use crate::drone::utils::new_flood_response;
use crate::drone::RustyDrone;
use crate::{extract, extract_mut};
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{NodeType, Packet, PacketType};

impl RustyDrone {
    pub(super) fn handle_flood_request(&self, packet: Packet, already_rec: bool) {
        if already_rec {
            if let Some(response_packet) = self.respond_old_flood(packet) {
                self.send_packet(response_packet);
            }
        } else {
            // Technically it is possible to receive a packet with wrong data and not
            // knowing to who not send
            if let Some((response_packet, previous_hop)) = self.respond_new_flood(packet) {
                self.flood_packet(response_packet, previous_hop);
            }
        }
    }

    /// need to create flood response
    pub(super) fn respond_old_flood(&self, packet: Packet) -> Option<Packet> {
        let mut flood_res = new_flood_response(extract!(packet.pack_type, PacketType::FloodRequest).unwrap());

        flood_res.path_trace.push((self.id, NodeType::Drone));
        let hops = flood_res
            .path_trace
            .iter()
            .map(|(node_id, _)| *node_id)
            .rev()
            .collect::<Vec<_>>();

        Some(Packet::new_flood_response(
            SourceRoutingHeader { hop_index: 1, hops },
            packet.session_id,
            flood_res
        ))
    }

    /// need to update flood request
    pub(super) fn respond_new_flood(&self, mut packet: Packet) -> Option<(Packet, Option<NodeId>)> {
        let flood = extract_mut!(packet.pack_type, PacketType::FloodRequest).unwrap();

        let prev_hop = flood.path_trace.last().map(|x| x.0);
        flood.path_trace.push((self.id, NodeType::Drone));
        Some((packet, prev_hop))
    }
}
