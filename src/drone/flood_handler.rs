use crate::drone::RustyDrone;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{FloodRequest, FloodResponse, NodeType, Packet};

impl RustyDrone {
    /// Handle the processing of flood-request packets.
    pub(super) fn respond_flood_request(&mut self, session_id: u64, flood: &FloodRequest) {
        let no_other_neighbours = self.packet_send.len() == 1;

        if self.already_received_flood(flood) || no_other_neighbours {
            // Flood request is already seen or we have no one to forward it to, it should be terminated.
            self.respond_old(session_id, flood);
        } else {
            // Flood request should be forwarded.
            self.respond_new(session_id, flood);
        }
    }

    /// Handle flood request termination, sending back a flood response.
    fn respond_old(&self, session_id: u64, request: &FloodRequest) {
        let mut new_path = request.path_trace.clone();
        // Add ourselves to the path
        new_path.push((self.id, NodeType::Drone));

        // Use reverse of the flood path as routing
        let mut hops = new_path
            .iter()
            .map(|(node_id, _)| *node_id)
            .rev()
            .collect::<Vec<_>>();

        // Add the request initiator to the routing path
        // (in case they did not add themselves when starting the request)
        if hops.last() != Some(&request.initiator_id) {
            hops.push(request.initiator_id);
        }

        // Send back flood response
        self.send_to_next(Packet::new_flood_response(
            SourceRoutingHeader { hop_index: 1, hops },
            session_id,
            FloodResponse {
                flood_id: request.flood_id,
                path_trace: new_path,
            },
        ));
    }

    /// Flood request has not finished yet, forward it to all our neighbors
    /// (excluding the one that send it to us).
    fn respond_new(&self, session_id: u64, flood: &FloodRequest) {
        // Exclude the neighbor we received the packet from in the forward
        // Fall back on initiator id in case path_trace is empty
        let prev_hop = flood.path_trace.last().map_or(flood.initiator_id, |x| x.0);

        let mut new_flood = flood.clone();
        // Add ourselves to the path
        new_flood.path_trace.push((self.id, NodeType::Drone));

        // Forward to our neighbors
        self.flood_except(
            prev_hop,
            &Packet::new_flood_request(
                SourceRoutingHeader {
                    hop_index: 0,
                    hops: vec![],
                },
                session_id,
                new_flood,
            ),
        );
    }
}
