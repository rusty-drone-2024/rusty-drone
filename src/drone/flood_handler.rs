use crate::drone::RustyDrone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{FloodResponse, NodeType, Packet, PacketType};

impl RustyDrone {
    pub(super) fn handle_flood_request(&self, packet: Packet, already_rec: bool) {
        if already_rec || self.packet_send.len() <= 1 {
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
        let mut flood;
        if let PacketType::FloodRequest(ref flood_ref) = packet.pack_type {
            flood = flood_ref.clone();
        } else {
            // Should not happen (i know it is SHIT)
            return None;
        }

        flood.path_trace.push((self.id, NodeType::Drone));
        let hops = flood
            .path_trace
            .iter()
            .map(|(node_id, _)| *node_id)
            .rev()
            .collect::<Vec<_>>();

        Some(Packet::new_flood_response(
            SourceRoutingHeader { hop_index: 1, hops },
            packet.session_id,
            FloodResponse {
                flood_id: flood.flood_id,
                path_trace: flood.path_trace,
            },
        ))
    }

    /// need to update flood request
    pub(super) fn respond_new_flood(&self, mut packet: Packet) -> Option<(Packet, NodeId)> {
        let flood;
        if let PacketType::FloodRequest(ref mut flood_ref) = packet.pack_type {
            flood = flood_ref;
        } else {
            // Should not happen (i know it is SHIT)
            return None;
        }

        let prev_hop = flood.path_trace.last()?.0;
        flood.path_trace.push((self.id, NodeType::Drone));
        Some((packet, prev_hop))
    }
}
