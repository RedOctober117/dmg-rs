pub mod dmg;

pub fn main() {
    println!("{:?}", u8::MAX.overflowing_shl(1));
}
