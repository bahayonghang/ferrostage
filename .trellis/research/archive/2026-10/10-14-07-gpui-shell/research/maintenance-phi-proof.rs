// One-shot toolchain evidence, not a product regression test.
fn main() {
    let literal = 1.618_034_f32.to_bits();
    let constant = std::f32::consts::PHI.to_bits();
    println!("literal={literal:#010x}; PHI={constant:#010x}; equal={}", literal == constant);
    assert_eq!(literal, constant);
}
