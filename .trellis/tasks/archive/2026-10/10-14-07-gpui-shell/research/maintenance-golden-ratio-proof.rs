// One-shot toolchain evidence, not a product regression test.
fn main() {
    let literal = 1.618_034_f32.to_bits();
    let constant = std::f32::consts::GOLDEN_RATIO.to_bits();
    println!(
        "literal={literal:#010x}; GOLDEN_RATIO={constant:#010x}; equal={}",
        literal == constant
    );
    assert_eq!(literal, constant);
}
