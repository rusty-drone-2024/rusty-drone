use wg_2024::controller::DroneEvent;
use wg_2024::packet::{Packet, PacketType};
use wg_2024::packet::NackType::{DestinationIsDrone, Dropped, ErrorInRouting, UnexpectedRecipient};
use crate::drone::RustyDrone;



// Command/packets handling part
impl RustyDrone {
    /// return the response to be sent
    pub(super) fn handle_packet(&mut self, packet: Packet, crashing: bool) {
        // Do custom handling for floods
        if let PacketType::FloodRequest(ref flood) = packet.pack_type {
            if !crashing {
                let already_rec = self.already_received_flood(
                    flood.flood_id,
                    flood.initiator_id,
                    packet.session_id,
                );
                self.handle_flood_request(packet, already_rec);
            }
        } else {
            let res = self.respond_normal_types(packet, crashing);
            if let Some(response_packet) = res {
                self.send_packet(response_packet);
            }
        }
    }
    
    /// Return wheter it should crash or not
    pub(super) fn respond_normal_types(&self, mut packet: Packet, crashing: bool) -> Option<Packet> {
        let droppable = matches!(packet.pack_type, PacketType::MsgFragment(_));
        let routing = &mut packet.routing_header;

        // If unexpected packets
        if routing.current_hop() != Some(self.id) {
            // the protocol say so but it is just dumb
            return self.create_nack(packet, UnexpectedRecipient(self.id), droppable, true);
        }

        if routing.is_last_hop() {
            // cannot nack only fragment, rest will be dropped
            return self.create_nack(packet, DestinationIsDrone, droppable, false);
        }

        // next hop must exist
        let next_hop = routing.next_hop()?;
        if !self.packet_send.contains_key(&next_hop) {
            return self.create_nack(packet, ErrorInRouting(next_hop), droppable, true);
        }

        if droppable && self.should_drop() {
            let _ = self
                .controller_send
                .send(DroneEvent::PacketDropped(packet.clone()));
            return self.create_nack(packet, Dropped, droppable, false);
        }

        if crashing && droppable {
            let current_hop = routing.current_hop()?;
            return self.create_nack(packet, ErrorInRouting(current_hop), droppable, false);
        }

        // forward
        // cannot be done before as asked by the protocol (should be before .is_last_hop)
        routing.increase_hop_index();
        Some(packet)
    }
}
