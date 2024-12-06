use crate::testing_utils::data::new_test_fragment_packet;
use crate::testing_utils::{test_initialization_with_value, test_muliple_initialization};
use crossbeam_channel::unbounded;
use wg_2024::controller::{DroneCommand, DroneEvent};

#[test]
fn test_drone_packet_fragment_old() {
    let mut drones = test_muliple_initialization(3);
    let (_options1, mut drone1) = drones.pop().unwrap();
    let (options2, mut drone2) = drones.pop().unwrap();
    let (options3, drone3) = drones.pop().unwrap();

    assert!(!drone1.handle_commands(DroneCommand::AddSender(
        drone2.id,
        options2.packet_drone_in.clone()
    )));
    assert!(!drone2.handle_commands(DroneCommand::AddSender(
        drone3.id,
        options3.packet_drone_in.clone()
    )));

    let packet = new_test_fragment_packet(&[drone1.id, drone2.id, drone3.id]);

    drone2.handle_packet(packet.clone(), false);
    let drone2_event = options2.event_recv.try_recv().unwrap();
    match drone2_event {
        DroneEvent::PacketSent(sent_packet) => {
            assert_eq!(packet.session_id, sent_packet.session_id)
        }
        _ => assert!(false),
    }
}

#[test]
fn test_drone_packet_forward() {
    let (options, mut drone) = test_initialization_with_value(11, 0.0);

    let (new_sender, new_receiver) = unbounded();
    let mut packet = new_test_fragment_packet(&[10, 11, 12]);

    drone.handle_commands(DroneCommand::AddSender(12, new_sender));
    drone.handle_packet(packet.clone(), false);

    (&mut packet.routing_header).increase_hop_index();

    let forwarded_packet = new_receiver.try_recv().unwrap();
    assert_eq!(packet.clone(), forwarded_packet);

    let event = options.event_recv.try_recv().unwrap();
    assert_eq!(event, DroneEvent::PacketSent(packet));
}
