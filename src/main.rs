pub mod dmg;

pub fn main() {
    println!("{:?}", 0xFF_u8.saturating_add(0x03_u8));
}
