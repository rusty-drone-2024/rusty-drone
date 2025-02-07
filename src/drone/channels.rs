use crate::drone::RustyDrone;
use wg_2024::controller::DroneEvent::{ControllerShortcut, PacketDropped, PacketSent};
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;

impl RustyDrone {
    /// Send packet to the next node in the routing header.
    pub fn send_to_next(&self, packet: Packet) {
        let Some(next_hop) = packet.routing_header.current_hop() else {
            return;
        };

        let Some(channel) = self.packet_send.get(&next_hop) else {
            return;
        };

        let _ = channel.send(packet.clone());
        let _ = self.controller_send.send(PacketSent(packet));
    }

    /// Forward packet to all neighbors except previous hop.
    pub(super) fn flood_except(&self, previous_hop: NodeId, packet: &Packet) {
        for (node_id, channel) in &self.packet_send {
            if *node_id != previous_hop {
                let _ = channel.send(packet.clone());
                #[cfg(feature = "packet_sent_for_flood")]
                let _ = self.controller_send.send(PacketSent(packet.clone()));
            }
        }
    }

    /// Send packet over shortcut chanel.
    pub(super) fn use_shortcut(&self, packet: Packet) {
        let _ = self.controller_send.send(ControllerShortcut(packet));
    }

    /// Inform Simulation Controller that a packet was dropped.
    pub(super) fn notify_dropped(&self, packet: Packet) {
        let _ = self.controller_send.send(PacketDropped(packet));
    }
}
