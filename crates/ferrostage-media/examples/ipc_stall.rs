//! Adversarial child for product supervisor tests: never acknowledges Stop.
use std::{io::Write, time::Duration};
fn main() {
    if ferrostage_media::protocol::read(&mut std::io::stdin().lock()).is_err() {
        return;
    }
    let mut stderr = std::io::stderr().lock();
    for _ in 0..2048 {
        if stderr.write_all(&[b'x'; 4096]).is_err() {
            break;
        }
    }
    drop(stderr);
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
