use crate::checkers::flood::assert_topology_on_client;
use crate::checkers::TIMEOUT;
use crate::testing_utils::data::new_flood_request;
use crate::testing_utils::Network;
use wg_2024::packet::NodeType;

#[test]
fn test_matrix_loop_flood() {
    let net = Network::create_and_run(
        19,
        &[
            (0, 1),

            (1, 2),
            (2, 3),
            (3, 4),
            (4, 5),
            (5, 6),
            (6, 7),
            (6, 8),
            (8, 9),
            
            (10, 1),
            (11, 2),
            (12, 3),
            (13, 4),
            (14, 5),
            (15, 6),
            (16, 7),
            (17, 8),
            (18, 9),

            (10, 11),
            (11, 12),
            (12, 13),
            (13, 14),
            (14, 15),
            (15, 16),
            (16, 17),
            (17, 18),
        ],
        &[0],
    );

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    assert_topology_on_client(
        net,
        vec![
            (1, NodeType::Drone),
            (2, NodeType::Drone),
            (3, NodeType::Drone),
            (4, NodeType::Drone),
            (5, NodeType::Drone),
            (6, NodeType::Drone),
            (7, NodeType::Drone),
            (8, NodeType::Drone),
            (9, NodeType::Drone),
            (10, NodeType::Drone),
            (11, NodeType::Drone),
            (12, NodeType::Drone),
            (13, NodeType::Drone),
            (14, NodeType::Drone),
            (15, NodeType::Drone),
            (16, NodeType::Drone),
            (17, NodeType::Drone),
            (18, NodeType::Drone),
        ],
        TIMEOUT * 5,
    );
}

#[test]
fn test_star_loop_flood() {
    let net = Network::create_and_run(
        11,
        &[
            (0, 1),
                
            (1, 4),
            (2, 5),
            (3, 6),
            (4, 7),
            (5, 8),
            (6, 9),
            (7, 10),
            (8, 1),
            (9, 2),
            (10, 3)
        ],
        &[0],
    );

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    assert_topology_on_client(
        net,
        vec![
            (1, NodeType::Drone),
            (2, NodeType::Drone),
            (3, NodeType::Drone),
            (4, NodeType::Drone),
            (5, NodeType::Drone),
            (6, NodeType::Drone),
            (7, NodeType::Drone),
            (8, NodeType::Drone),
            (9, NodeType::Drone),
            (10, NodeType::Drone),
        ],
        TIMEOUT * 5,
    );
}

/*
#[test]
fn test_butterfly_loop_flood() {
    let net = Network::create_and_run(
        13,
        &[
            (0, 1),

            (1, 2),
            (1, 3),
            (1, 4),
            (2, 3),
            (2, 4),
            (3, 4),

            (5, 6),
            (5, 7),
            (5, 8),
            (6, 7),
            (6, 8),
            (7, 8),

            (9, 10),
            (9, 11),
            (9, 12),
            (10, 11),
            (10, 12),
            (11, 12),
        ],
        &[0],
    );

    let flood = new_flood_request(5, 7, 0, false);
    net.send_to_dest_as_client(0, 1, flood).unwrap();

    assert_topology_on_client(
        net,
        vec![
            (1, NodeType::Drone),
            (2, NodeType::Drone),
            (3, NodeType::Drone),
            (4, NodeType::Drone),
            (5, NodeType::Drone),
            (6, NodeType::Drone),
            (7, NodeType::Drone),
            (8, NodeType::Drone),
            (9, NodeType::Drone),
            (10, NodeType::Drone),
            (11, NodeType::Drone),
            (12, NodeType::Drone),
        ],
        TIMEOUT * 5,
    );
}
*/
