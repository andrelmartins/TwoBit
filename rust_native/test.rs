use std::mem;

fn main() {
  let v = [1u8, 0u8,0u8,1u8];
  let z : u32 = unsafe{ mem::transmute(v) };
  println!("{}", z);
}
