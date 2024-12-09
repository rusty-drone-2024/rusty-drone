use crate::drone::RustyDrone;
use wg_2024::controller::DroneEvent;
use wg_2024::packet::NackType::{DestinationIsDrone, Dropped, ErrorInRouting, UnexpectedRecipient};
use wg_2024::packet::{Packet, PacketType};

// Command/packets handling part
impl RustyDrone {
    pub(super) fn respond_normal(&self, packet: &Packet, crashing: bool) {
        let res = self.respond_normal_int(packet, crashing);
        if let Some(ref response_packet) = res {
            self.send_normal_packet(response_packet);
        }
    }

    /// Return wheter it should crash or not
    fn respond_normal_int(&self, packet: &Packet, crashing: bool) -> Option<Packet> {
        let mut packet = packet.clone();
        let droppable = matches!(packet.pack_type, PacketType::MsgFragment(_));
        let routing = &mut packet.routing_header;

        // If unexpected packets
        if routing.current_hop() != Some(self.id) {
            packet.routing_header.hops[packet.routing_header.hop_index] = self.id;
            return self.create_nack(&packet, UnexpectedRecipient(self.id), droppable, true);
        }

        if crashing && droppable {
            let current_hop = routing.current_hop()?;
            return self.create_nack(&packet, ErrorInRouting(current_hop), droppable, false);
        }

        if routing.is_last_hop() {
            // cannot nack only fragment, rest will be dropped
            return self.create_nack(&packet, DestinationIsDrone, droppable, false);
        }

        // next hop must exist
        let next_hop = routing.next_hop()?;
        if !self.packet_send.contains_key(&next_hop) {
            return self.create_nack(&packet, ErrorInRouting(next_hop), droppable, true);
        }

        if droppable && self.should_drop() {
            let _ = self
                .controller_send
                .send(DroneEvent::PacketDropped(packet.clone()));
            return self.create_nack(&packet, Dropped, droppable, false);
        }

        // forward
        // cannot be done before as asked by the protocol (should be before .is_last_hop)
        routing.increase_hop_index();
        Some(packet)
    }
}
