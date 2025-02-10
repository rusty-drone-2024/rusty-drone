use crate::drone::{utils, RustyDrone};
use wg_2024::packet::NackType::{DestinationIsDrone, Dropped, ErrorInRouting, UnexpectedRecipient};
use wg_2024::packet::{Nack, NackType, Packet, PacketType};

impl RustyDrone {
    /// Handle the processing of non-flood-request packets.
    pub(super) fn respond_normal(&self, packet: &Packet, crashing: bool) {
        let droppable = matches!(packet.pack_type, PacketType::MsgFragment(_));
        let routing = &packet.routing_header;

        // We received this packet, but according to the routing header, we are not the current node on the path
        if routing.current_hop() != Some(self.id) {
            self.nack_packet(packet, UnexpectedRecipient(self.id), droppable, true);
            return;
        }

        // We received a droppable packet as a crashed drone
        if crashing && droppable {
            self.nack_packet(packet, ErrorInRouting(self.id), droppable, false);
            return;
        }

        match routing.next_hop() {
            None => {
                // This packet was send to us, but drones are not valid end destinations
                if droppable {
                    self.nack_packet(packet, DestinationIsDrone, droppable, false);
                }
                return;
            }
            Some(next) => {
                if !self.packet_send.contains_key(&next) {
                    // We do not have a connection to the next node that should receive this packet
                    self.nack_packet(packet, ErrorInRouting(next), droppable, true);
                    return;
                }
            }
        }

        if droppable && self.should_drop() {
            // Packet got dropped by packet drop rate
            self.notify_dropped(packet.clone());
            self.nack_packet(packet, Dropped, droppable, false);
            return;
        }

        // Forward packet to the next node in the route (one of our neighbors)
        self.forward_packet(packet);
    }

    /// Send packet to the next node in the packet route.
    fn forward_packet(&self, packet: &Packet) {
        let mut routing_header = packet.routing_header.clone();

        // Set the current hop of the route to the next node
        routing_header.increase_hop_index();

        self.send_to_next(Packet {
            routing_header,
            session_id: packet.session_id,
            pack_type: packet.pack_type.clone(),
        });
    }

    /// Send nack in response to received packet.
    fn nack_packet(
        &self,
        packet: &Packet,
        nack_type: NackType,
        droppable: bool,
        shortcuttable: bool,
    ) {
        if !droppable {
            if shortcuttable {
                // Packet cannot be dropped, sending through shortcut
                let mut routing_header = packet.routing_header.clone();
                routing_header.increase_hop_index();

                self.use_shortcut(Packet {
                    routing_header,
                    session_id: packet.session_id,
                    pack_type: packet.pack_type.clone(),
                });
            }
            return;
        }

        // Send nack to the first hop
        self.send_to_next(Packet::new_nack(
            self.get_routing_back(&packet.routing_header),
            packet.session_id,
            Nack {
                nack_type,
                fragment_index: utils::get_fragment_index(&packet.pack_type),
            },
        ));
    }
}
