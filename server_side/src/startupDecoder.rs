use std::str;

#[derive(Debug, PartialEq)]
pub struct StartupMessage {
   pub  tag: u8,
   pub  user: String,
   pub  database: String,
}

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    TooShort,
    LengthMismatch { declared: i32, actual: usize },
    BadTag(u8),
    MissingTerminator,
    InvalidUtf8,
    UnexpectedKey(String),
}

/// Reads one null-terminated string; returns it and the remaining bytes.
fn read_cstr(buf: &[u8]) -> Result<(&str, &[u8]), DecodeError> {
    let end = buf
        .iter()
        .position(|&b| b == 0)
        .ok_or(DecodeError::MissingTerminator)?;
    let s = str::from_utf8(&buf[..end]).map_err(|_| DecodeError::InvalidUtf8)?;
    Ok((s, &buf[end + 1..]))
}

pub fn split_startup(buf: &[u8]) -> Result<StartupMessage, DecodeError> {
    // length (4) + tag (1) + final terminator (1) = minimum 6 bytes
    if buf.len() < 6 {
        return Err(DecodeError::TooShort);
    }

    // 1. Length prefix (counts its own 4 bytes)
    let declared = i32::from_be_bytes(buf[0..4].try_into().unwrap());
    if declared < 0 || declared as usize != buf.len() {
        return Err(DecodeError::LengthMismatch { declared, actual: buf.len() });
    }

    // 2. Tag byte
    let tag = buf[4];
    if tag != b'S' {
        return Err(DecodeError::BadTag(tag));
    }

    // 3. Strip the final parameter-list terminator
    let params = match buf[5..].split_last() {
        Some((0, params)) => params,
        _ => return Err(DecodeError::MissingTerminator),
    };

    // 4. Read "user\0<value>\0" then "database\0<value>\0"
    let (key, rest) = read_cstr(params)?;
    if key != "user" {
        return Err(DecodeError::UnexpectedKey(key.to_string()));
    }
    let (user, rest) = read_cstr(rest)?;

    let (key, rest) = read_cstr(rest)?;
    if key != "database" {
        return Err(DecodeError::UnexpectedKey(key.to_string()));
    }
    let (database, _rest) = read_cstr(rest)?;

    Ok(StartupMessage {
        tag,
        user: user.to_string(),
        database: database.to_string(),
    })
}