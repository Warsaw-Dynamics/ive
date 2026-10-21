//! Frames between a host and its client: a tag byte, a big-endian u32 payload length, the payload.

use std::io::{self, Read, Write};

/// Larger frames are refused rather than allocated.
const MAX_PAYLOAD: u32 = 16 * 1024 * 1024;

const INPUT: u8 = 1;
const RESIZE: u8 = 2;
const OUTPUT: u8 = 3;
const EXITED: u8 = 4;
const ATTACHED: u8 = 5;
const REPLAY: u8 = 6;
const HANGUP: u8 = 7;

/// Version of these frames; a client and host that differ do not talk.
///
/// Hosts keep running the binary that started them, and a Restart connects a newly built IVE to them. Changing
/// the frames or bumping this version leaves every terminal started by an older build unattachable until it ends,
/// so keep new frames compatible with old hosts, or accept that cost knowingly.
pub const PROTOCOL: u16 = 1;

/// Terminal size in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub columns: u16,
    pub lines: u16,
}

impl Size {
    pub const DEFAULT: Size = Size { columns: 80, lines: 24 };

    /// Parses `80x24`.
    pub fn parse(text: &str) -> io::Result<Size> {
        let parsed = text.split_once('x').and_then(|(columns, lines)| Some(Size { columns: columns.parse().ok()?, lines: lines.parse().ok()? }));
        match parsed {
            Some(size) if size.columns > 0 && size.lines > 0 => Ok(size),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("size must look like 80x24, got {text:?}"))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// client to host: bytes typed into the program
    Input(Vec<u8>),
    /// client to host: the client's terminal size; also asks the program to redraw
    Resize(Size),
    /// host to client: bytes the program wrote
    Output(Vec<u8>),
    /// host to client: the program exited with this code, 128 + signal when killed
    Exited(i32),
    /// host to client, first on every connection
    Attached { protocol: u16, program_pid: u32 },
    /// host to client, after the client's first `Resize`: recent output, already answered when it first ran, so
    /// terminal queries in it must not be answered again
    Replay(Vec<u8>),
    /// client to host: end the program, as closing its terminal would
    HangUp,
}

impl Frame {
    pub fn write_to(&self, out: &mut impl Write) -> io::Result<()> {
        let mut fixed = [0u8; 6];
        let (tag, payload): (u8, &[u8]) = match self {
            Frame::Input(bytes) => (INPUT, bytes),
            Frame::Output(bytes) => (OUTPUT, bytes),
            Frame::Replay(bytes) => (REPLAY, bytes),
            Frame::HangUp => (HANGUP, &[]),
            Frame::Attached { protocol, program_pid } => {
                fixed[..2].copy_from_slice(&protocol.to_be_bytes());
                fixed[2..].copy_from_slice(&program_pid.to_be_bytes());
                (ATTACHED, &fixed)
            }
            Frame::Resize(size) => {
                fixed[..2].copy_from_slice(&size.columns.to_be_bytes());
                fixed[2..4].copy_from_slice(&size.lines.to_be_bytes());
                (RESIZE, &fixed[..4])
            }
            Frame::Exited(code) => {
                fixed[..4].copy_from_slice(&code.to_be_bytes());
                (EXITED, &fixed[..4])
            }
        };
        let mut frame = Vec::with_capacity(5 + payload.len());
        frame.push(tag);
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(payload);
        out.write_all(&frame)
    }

    pub fn read_from(input: &mut impl Read) -> io::Result<Frame> {
        let mut header = [0u8; 5];
        input.read_exact(&mut header)?;
        let length = u32::from_be_bytes([header[1], header[2], header[3], header[4]]);
        if length > MAX_PAYLOAD {
            return Err(invalid(format!("frame of {length} bytes exceeds the {MAX_PAYLOAD} byte limit")));
        }
        let mut payload = vec![0u8; length as usize];
        input.read_exact(&mut payload)?;
        decode(header[0], payload)
    }
}

fn decode(tag: u8, payload: Vec<u8>) -> io::Result<Frame> {
    match (tag, payload.len()) {
        (INPUT, _) => Ok(Frame::Input(payload)),
        (OUTPUT, _) => Ok(Frame::Output(payload)),
        (REPLAY, _) => Ok(Frame::Replay(payload)),
        (HANGUP, 0) => Ok(Frame::HangUp),
        (ATTACHED, 6) => Ok(Frame::Attached {
            protocol: u16::from_be_bytes([payload[0], payload[1]]),
            program_pid: u32::from_be_bytes([payload[2], payload[3], payload[4], payload[5]]),
        }),
        (RESIZE, 4) => Ok(Frame::Resize(Size {
            columns: u16::from_be_bytes([payload[0], payload[1]]),
            lines: u16::from_be_bytes([payload[2], payload[3]]),
        })),
        (EXITED, 4) => Ok(Frame::Exited(i32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]))),
        _ => Err(invalid(format!("unknown frame: tag {tag}, {} payload bytes", payload.len()))),
    }
}

fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_read_back_every_frame_written() {
        // given
        let frames = [
            Frame::Input(b"ls\n".to_vec()),
            Frame::Resize(Size { columns: 120, lines: 40 }),
            Frame::Output(Vec::new()),
            Frame::Exited(-1),
            Frame::Attached { protocol: 1, program_pid: 42 },
            Frame::Replay(b"x".to_vec()),
            Frame::HangUp,
        ];
        let mut bytes = Vec::new();
        for frame in &frames {
            frame.write_to(&mut bytes).unwrap();
        }

        // when
        let mut input = bytes.as_slice();
        let read: Vec<Frame> = frames.iter().map(|_| Frame::read_from(&mut input).unwrap()).collect();

        // then
        assert_eq!(read, frames);
        assert!(input.is_empty());
    }

    #[test]
    fn should_refuse_oversized_frame() {
        // given
        let mut bytes = vec![OUTPUT];
        bytes.extend_from_slice(&(MAX_PAYLOAD + 1).to_be_bytes());

        // when
        let error = Frame::read_from(&mut bytes.as_slice()).unwrap_err();

        // then
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
