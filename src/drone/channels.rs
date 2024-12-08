use crate::drone::RustyDrone;
use wg_2024::controller::DroneEvent;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

impl RustyDrone {
    pub(super) fn use_shortcut(&self, packet: Packet) {
        let _ = self
            .controller_send
            .send(DroneEvent::ControllerShortcut(packet));
    }

    pub(super) fn send_packet(&self, packet: Packet) {
        let next = packet.routing_header.current_hop();
        if let Some(next_hop) = next {
            if let Some(channel) = self.packet_send.get(&next_hop) {
                let _ = channel.send(packet.clone());
                let _ = self.controller_send.send(DroneEvent::PacketSent(packet));
            }
        }
        // Ignore broken send (it is an internal problem)
    }

    pub(super) fn flood_packet(&self, packet: Packet, previous_hop: NodeId) {
        for (node_id, channel) in self.packet_send.iter() {
            if *node_id != previous_hop {
                let _ = channel.send(packet.clone());
            }
        }
    }
}
