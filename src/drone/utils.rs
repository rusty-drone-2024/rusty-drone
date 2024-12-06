use rand::Rng;
use wg_2024::packet::PacketType;

pub fn should_drop(pdr: f32) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..1.0) < pdr
}

pub fn get_fragment_index(packet_type: PacketType) -> u64 {
    if let PacketType::MsgFragment(f) = packet_type {
        return f.fragment_index;
    }
    0
}
