use crate::drone::RustyDrone;
use rand::Rng;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{FloodRequest, PacketType};

macro_rules! extract {
    ($e:expr, $p:path) => {
        match &$e {
            $p(ref value) => Some(value),
            _ => None,
        }
    };
}

impl RustyDrone {
    #[allow(deprecated)]
    /// Decides if this packet should be dropped according to the packet drop rate.
    pub(super) fn should_drop(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0) < self.pdr
    }

    /// Adds the flood request to the received flood requests, returns if the flood request was already present before.
    pub(super) fn already_received_flood(&mut self, flood: &FloodRequest) -> bool {
        !self
            .received_floods
            .insert((flood.flood_id, flood.initiator_id))
    }

    /// Calculates the route to send a packet back to the sender of a packet (according to its routing).
    /// Trims all nodes in the route after this one and reverses the routing order.
    pub(super) fn get_routing_back(&self, routing: &SourceRoutingHeader) -> SourceRoutingHeader {
        let mut hops = routing
            .hops
            .iter()
            .copied()
            .take(routing.hop_index + 1)
            .rev()
            .collect::<Vec<_>>();

        hops[0] = self.id;

        SourceRoutingHeader { hops, hop_index: 1 }
    }
}

/// Get fragment index of MsgFragment or use default fragment index 1 for other packet types.
pub(super) fn get_fragment_index(packet_type: &PacketType) -> u64 {
    extract!(packet_type, PacketType::MsgFragment).map_or(1, |x| x.fragment_index)
}
