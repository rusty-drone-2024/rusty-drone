use crate::RustyDrone;
use rusty_tester::*;

#[allow(dead_code)]
type Tested = RustyDrone;

#[test]
fn test_drone_destination_is_drone(){
    general::test_drone_destination_is_drone::<Tested>();
}

#[test]
fn test_drone_error_in_routing(){
    general::test_drone_error_in_routing::<Tested>();
}

#[test]
fn test_drone_packet_1_hop(){
    general::test_drone_packet_1_hop::<Tested>();
}

#[test]
fn test_drone_packet_3_hop(){
    general::test_drone_packet_3_hop::<Tested>();
}

#[test]
fn test_drone_packet_3_hop_crash(){
    general::test_drone_packet_3_hop_crash::<Tested>();
}

#[test]
fn test_drone_packet_255_hop(){
    general::test_drone_packet_255_hop::<Tested>();
}

#[test]
fn test_easiest_flood(){
    flood::normal_flood::test_easiest_flood::<Tested>();
}

#[test]
fn test_loop_flood(){
    flood::normal_flood::test_loop_flood::<Tested>();
}

#[test]
fn test_hard_loop_flood(){
    flood::normal_flood::test_hard_loop_flood::<Tested>();
}

#[test]
fn test_matrix_loop_flood(){
    flood::extra_flood::test_matrix_loop_flood::<Tested>();
}

#[test]
fn test_star_loop_flood(){
    flood::extra_flood::test_star_loop_flood::<Tested>();
}

#[test]
fn test_butterfly_loop_flood(){
    flood::extra_flood::test_butterfly_loop_flood::<Tested>();
}



#[test]
fn test_tree_loop_flood(){
    flood::extra_flood::test_tree_loop_flood::<Tested>();
}