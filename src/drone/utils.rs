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
    pub(super) fn should_drop(&self) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0) < self.pdr
    }

    pub(super) fn already_received_flood(&mut self, flood: &FloodRequest) -> bool {
        // TODO talk with WG
        !self
            .received_floods
            .insert((flood.flood_id, flood.initiator_id))
    }

    pub(super) fn get_routing_back(&self, routing: &SourceRoutingHeader) -> SourceRoutingHeader {
        let mut hops = routing
            .hops
            .iter()
            .copied()
            .take(routing.hop_index + 1)
            .rev()
            .collect::<Vec<_>>();

        hops[0] = self.id; //TODO packet.routing_header.hops[packet.routing_header.hop_index] = self.id;

        SourceRoutingHeader { hops, hop_index: 1 }
    }
}

pub(super) fn get_fragment_index(packet_type: &PacketType) -> u64 {
    extract!(packet_type, PacketType::MsgFragment).map_or(1, |x| x.fragment_index)
}
