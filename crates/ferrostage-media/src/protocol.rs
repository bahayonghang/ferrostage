//! Versioned, bounded binary messages. Validate fixed headers before allocating.
use crate::{MAX_HEIGHT, MAX_WIDTH, MediaError};
use ferrostage_core::BgraFrame;
use std::io::{Read, Write};
const HEADER: usize = 40;
const MAGIC: &[u8; 4] = b"FSM1";
const MAX_PATH: usize = 32768;
#[derive(Clone, Debug)]
pub struct Open {
    pub path: String,
    pub max_height: u32,
    pub start_ms: u64,
    pub duration_ms: Option<u64>,
    /// Total plays; zero means forever.
    pub plays: u32,
    pub silent: bool,
}
#[derive(Clone, Debug)]
pub enum Packet {
    Open(Open),
    Pause(bool),
    Stop,
    Frame {
        sequence: u64,
        position_ms: u64,
        duration_ms: u64,
        pixels: BgraFrame,
    },
    Status {
        position_ms: u64,
        duration_ms: u64,
        paused: bool,
    },
    End,
    Error(MediaError),
}
fn code(error: MediaError) -> u32 {
    match error {
        MediaError::RuntimeUnavailable => 1,
        MediaError::SourceUnavailable => 2,
        MediaError::Unsupported => 3,
        MediaError::Limit => 4,
        MediaError::Protocol => 5,
        MediaError::Process => 6,
        MediaError::Timeout => 7,
        MediaError::Cancelled => 8,
    }
}
fn error(code: u32) -> Result<MediaError, MediaError> {
    Ok(match code {
        1 => MediaError::RuntimeUnavailable,
        2 => MediaError::SourceUnavailable,
        3 => MediaError::Unsupported,
        4 => MediaError::Limit,
        5 => MediaError::Protocol,
        6 => MediaError::Process,
        7 => MediaError::Timeout,
        8 => MediaError::Cancelled,
        _ => return Err(MediaError::Protocol),
    })
}
fn u32_at(h: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([h[i], h[i + 1], h[i + 2], h[i + 3]])
}
fn u64_at(h: &[u8], i: usize) -> u64 {
    u64::from(u32_at(h, i)) | (u64::from(u32_at(h, i + 4)) << 32)
}
fn put32(h: &mut [u8], i: usize, n: u32) {
    h[i..i + 4].copy_from_slice(&n.to_le_bytes());
}
fn put64(h: &mut [u8], i: usize, n: u64) {
    h[i..i + 8].copy_from_slice(&n.to_le_bytes());
}
pub fn write(writer: &mut impl Write, packet: &Packet) -> Result<(), MediaError> {
    let mut h = [0u8; HEADER];
    h[..4].copy_from_slice(MAGIC);
    let body: &[u8] = match packet {
        Packet::Open(open) => {
            if open.path.len() > MAX_PATH || !(1..=MAX_HEIGHT).contains(&open.max_height) {
                return Err(MediaError::Limit);
            }
            h[4] = 1;
            h[5] = u8::from(open.silent);
            put32(&mut h, 8, open.max_height);
            put32(&mut h, 12, open.plays);
            put64(&mut h, 16, open.start_ms);
            put64(&mut h, 24, open.duration_ms.unwrap_or(u64::MAX));
            open.path.as_bytes()
        }
        Packet::Pause(paused) => {
            h[4] = 2;
            h[5] = u8::from(*paused);
            &[]
        }
        Packet::Stop => {
            h[4] = 3;
            &[]
        }
        Packet::Frame {
            sequence,
            position_ms,
            duration_ms,
            pixels,
        } => {
            if pixels.width() > MAX_WIDTH || pixels.height() > MAX_HEIGHT {
                return Err(MediaError::Limit);
            }
            h[4] = 4;
            put32(&mut h, 8, pixels.width());
            put32(&mut h, 12, pixels.height());
            put64(&mut h, 16, *sequence);
            put64(&mut h, 24, *position_ms);
            put32(&mut h, 32, (*duration_ms).min(u64::from(u32::MAX)) as u32);
            pixels.pixels_bgra()
        }
        Packet::Status {
            position_ms,
            duration_ms,
            paused,
        } => {
            h[4] = 5;
            h[5] = u8::from(*paused);
            put64(&mut h, 16, *position_ms);
            put64(&mut h, 24, *duration_ms);
            &[]
        }
        Packet::End => {
            h[4] = 6;
            &[]
        }
        Packet::Error(e) => {
            h[4] = 7;
            put32(&mut h, 8, code(*e));
            &[]
        }
    };
    put32(&mut h, 36, body.len() as u32);
    writer
        .write_all(&h)
        .and_then(|()| writer.write_all(body))
        .and_then(|()| writer.flush())
        .map_err(|_| MediaError::Protocol)
}
pub fn read(reader: &mut impl Read) -> Result<Packet, MediaError> {
    let mut h = [0; HEADER];
    reader
        .read_exact(&mut h)
        .map_err(|_| MediaError::Protocol)?;
    if &h[..4] != MAGIC || h[5] > 1 || h[6] != 0 || h[7] != 0 {
        return Err(MediaError::Protocol);
    }
    let len = u32_at(&h, 36) as usize;
    let (width, height) = (u32_at(&h, 8), u32_at(&h, 12));
    match h[4] {
        1 if len > 0 && len <= MAX_PATH && (1..=MAX_HEIGHT).contains(&width) => {}
        4 if (1..=MAX_WIDTH).contains(&width)
            && (1..=MAX_HEIGHT).contains(&height)
            && len == width as usize * height as usize * 4 => {}
        2 | 3 | 5 | 6 | 7 if len == 0 => {}
        _ => return Err(MediaError::Protocol),
    }
    let mut body = vec![0; len];
    reader
        .read_exact(&mut body)
        .map_err(|_| MediaError::Protocol)?;
    Ok(match h[4] {
        1 => Packet::Open(Open {
            path: String::from_utf8(body).map_err(|_| MediaError::Protocol)?,
            max_height: width,
            start_ms: u64_at(&h, 16),
            duration_ms: (u64_at(&h, 24) != u64::MAX).then(|| u64_at(&h, 24)),
            plays: height,
            silent: h[5] != 0,
        }),
        2 => Packet::Pause(h[5] != 0),
        3 => Packet::Stop,
        4 => Packet::Frame {
            sequence: u64_at(&h, 16),
            position_ms: u64_at(&h, 24),
            duration_ms: u64::from(u32_at(&h, 32)),
            pixels: BgraFrame::new(width, height, width as usize * 4, body)
                .map_err(|_| MediaError::Protocol)?,
        },
        5 => Packet::Status {
            position_ms: u64_at(&h, 16),
            duration_ms: u64_at(&h, 24),
            paused: h[5] != 0,
        },
        6 => Packet::End,
        7 => Packet::Error(error(width)?),
        _ => return Err(MediaError::Protocol),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_hostile_header_before_body_allocation() {
        let mut h = [0; HEADER];
        h[..4].copy_from_slice(MAGIC);
        h[4] = 4;
        put32(&mut h, 8, u32::MAX);
        put32(&mut h, 12, u32::MAX);
        put32(&mut h, 36, u32::MAX);
        assert!(matches!(read(&mut h.as_slice()), Err(MediaError::Protocol)));
        h[4] = 1;
        put32(&mut h, 8, 1080);
        assert!(read(&mut h.as_slice()).is_err());
    }
    #[test]
    fn frame_roundtrip_preserves_bgra_and_identity_sequence() {
        let frame = BgraFrame::new(1, 1, 4, vec![9, 8, 7, 6]).unwrap();
        let mut bytes = vec![];
        write(
            &mut bytes,
            &Packet::Frame {
                sequence: 42,
                position_ms: 2000,
                duration_ms: 8000,
                pixels: frame.clone(),
            },
        )
        .unwrap();
        let Packet::Frame {
            sequence,
            position_ms,
            pixels,
            ..
        } = read(&mut bytes.as_slice()).unwrap()
        else {
            panic!()
        };
        assert_eq!((sequence, position_ms, pixels), (42, 2000, frame));
        bytes.pop();
        assert!(read(&mut bytes.as_slice()).is_err());
    }
}
