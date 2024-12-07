use crate::testing_utils::data::new_test_fragment_packet;
use crate::testing_utils::{test_initialization_with_value, test_muliple_initialization};
use crossbeam_channel::unbounded;
use wg_2024::controller::{DroneCommand, DroneEvent};

#[test]
fn test_drone_packet_forward() {
    let (options, mut drone) = test_initialization_with_value(11, 0.0);

    let (new_sender, new_receiver) = unbounded();
    drone.handle_commands(DroneCommand::AddSender(12, new_sender));
    
    let mut packet = new_test_fragment_packet(&[10, 11, 12]);
    drone.handle_packet(packet.clone(), false);

    let forwarded_packet = new_receiver.try_recv().unwrap();
    (&mut packet.routing_header).increase_hop_index();
    assert_eq!(packet.clone(), forwarded_packet);

    let event = options.event_recv.try_recv().unwrap();
    assert_eq!(event, DroneEvent::PacketSent(packet));
}


