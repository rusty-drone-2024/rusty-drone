use crate::drone::test::simple_drone_with_two_exit;
use crate::testing_utils::data::{new_flood_request, new_flood_request_with_path};
use wg_2024::packet::NodeType;

#[test]
fn test_drone_flood() {
    let packet = new_flood_request(5, 7, 10);
    let expected = new_flood_request_with_path(5, 7, 10, &[(11, NodeType::Drone)]);


    let (options, mut drone, packet_exit1, packet_exit2) = simple_drone_with_two_exit(11, 1.0, 12, 13);
    drone.handle_packet(packet.clone(), false);
    assert_eq!(expected, packet_exit1.try_recv().unwrap());
    assert_eq!(expected, packet_exit2.try_recv().unwrap());

    options.assert_expect_drone_event_fail();
}