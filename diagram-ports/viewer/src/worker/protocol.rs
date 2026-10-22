use serde::{Serialize, de::DeserializeOwned};
use std::io::{Read, Write};
pub const CONFIG: u8 = 1;
pub const SOURCE: u8 = 2;
pub const MEASURE: u8 = 3;
pub const ANSWER: u8 = 4;
pub const RESULT: u8 = 5;
pub const SVG: u8 = 6;
pub const ERROR: u8 = 7;
pub const HEADER_LIMIT: usize = 64 * 1024;
pub struct Budget(usize);
impl Budget {
    pub fn new(bytes: usize) -> Self {
        Self(bytes)
    }
}

pub fn write_frame(writer: &mut impl Write, tag: u8, bytes: &[u8]) -> Result<(), String> {
    writer
        .write_all(&[tag])
        .and_then(|_| writer.write_all(&(bytes.len() as u32).to_be_bytes()))
        .and_then(|_| writer.write_all(bytes))
        .and_then(|_| writer.flush())
        .map_err(|error| format!("cannot write worker protocol frame {tag}: {error}"))
}
pub fn write_json(writer: &mut impl Write, tag: u8, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("cannot encode worker protocol frame {tag}: {error}"))?;
    if bytes.len() > HEADER_LIMIT {
        return Err(format!(
            "worker metadata frame {tag} exceeds the {HEADER_LIMIT}-byte limit"
        ));
    }
    write_frame(writer, tag, &bytes)
}
pub fn read_frame(reader: &mut impl Read, budget: &mut Budget) -> Result<(u8, Vec<u8>), String> {
    let mut header = [0; 5];
    reader
        .read_exact(&mut header)
        .map_err(|error| format!("cannot read worker protocol header: {error}"))?;
    let length = u32::from_be_bytes(header[1..].try_into().unwrap()) as usize;
    let cap = match header[0] {
        SOURCE => 256 * 1024,
        SVG => 16 * 1024 * 1024,
        _ => HEADER_LIMIT,
    };
    if length > cap || length.saturating_add(5) > budget.0 {
        return Err(format!(
            "worker frame {} length {length} exceeds protocol output limit",
            header[0]
        ));
    }
    budget.0 -= length + 5;
    let mut bytes = vec![0; length];
    reader
        .read_exact(&mut bytes)
        .map_err(|error| format!("cannot read worker frame {} payload: {error}", header[0]))?;
    Ok((header[0], bytes))
}
pub fn read_expected(
    reader: &mut impl Read,
    tag: u8,
    budget: &mut Budget,
) -> Result<(u8, Vec<u8>), String> {
    let frame = read_frame(reader, budget)?;
    if frame.0 != tag {
        return Err(format!("expected worker frame {tag}, got {}", frame.0));
    }
    Ok(frame)
}
pub fn read_json<T: DeserializeOwned>(
    reader: &mut impl Read,
    tag: u8,
    budget: &mut Budget,
) -> Result<T, String> {
    let (_, bytes) = read_expected(reader, tag, budget)?;
    decode(&bytes)
}
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("invalid worker protocol JSON: {error}"))
}
