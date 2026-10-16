use ferrostage_media::{
    MediaError,
    protocol::{self, Packet},
};
use std::{io, process::ExitCode, sync::mpsc, thread};
fn run() -> Result<(), MediaError> {
    let mut stdin = io::stdin().lock();
    let Packet::Open(open) = protocol::read(&mut stdin)? else {
        return Err(MediaError::Protocol);
    };
    drop(stdin);
    let (tx, rx) = mpsc::sync_channel(4);
    thread::spawn(move || {
        let mut input = io::stdin().lock();
        loop {
            let packet = match protocol::read(&mut input) {
                Ok(packet) => packet,
                // Parent death closes stdin. Terminate even if native startup
                // is currently stalled; do not orphan an audio process.
                Err(_) => std::process::exit(2),
            };
            let stop = matches!(packet, Packet::Stop);
            if tx.send(packet).is_err() || stop {
                break;
            }
        }
    });
    let mut stdout = io::stdout().lock();
    let result = ferrostage_media_helper::native::play(open, rx, |packet| {
        protocol::write(&mut stdout, &packet)
    });
    if let Err(error) = result {
        protocol::write(&mut stdout, &Packet::Error(error))?;
    }
    result
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
