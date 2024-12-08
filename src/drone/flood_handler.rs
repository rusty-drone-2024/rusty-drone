use crate::drone::utils::new_flood_response;
use crate::drone::RustyDrone;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{FloodRequest, NodeType, Packet};

impl RustyDrone {
    pub(super) fn handle_flood_request(&mut self, session_id: u64, flood: &FloodRequest) {
        if self.already_received_flood(flood) {
            self.respond_old_flood(session_id, flood);
        } else {
            self.respond_new_flood(session_id, flood);
        }
    }

    /// need to create flood response
    pub(super) fn respond_old_flood(&self, session_id: u64, flood: &FloodRequest) {
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

        self.send_packet(&Packet::new_flood_response(
            SourceRoutingHeader { hop_index: 1, hops },
            session_id,
            flood_res,
        ));
    }

    /// need to update flood request
    pub(super) fn respond_new_flood<'a>(&self, session_id: u64, flood: &FloodRequest) {
        let prev_hop = flood
            .path_trace
            .last()
            .map(|x| x.0)
            .unwrap_or(flood.initiator_id);

        let mut new_flood = flood.clone();
        new_flood.path_trace.push((self.id, NodeType::Drone));

        self.flood_packet(&Packet::new_flood_request(
            SourceRoutingHeader { hop_index: 0, hops: vec!() },
            session_id,
            new_flood,
        ), prev_hop);
    }
}
