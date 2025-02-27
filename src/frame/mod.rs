//! `frame` module contains general Frame functionality.
use std::sync::atomic::{AtomicI16, Ordering};

use crate::compression::Compression;
use crate::frame::frame_response::ResponseBody;
pub use crate::frame::traits::*;
use crate::types::to_n_bytes;
use uuid::Uuid;

/// Number of stream bytes in accordance to protocol.
pub const STREAM_LEN: usize = 2;
/// Number of body length bytes in accordance to protocol.
pub const LENGTH_LEN: usize = 4;

pub mod events;
pub mod frame_auth_challenge;
pub mod frame_auth_response;
pub mod frame_auth_success;
pub mod frame_authenticate;
pub mod frame_batch;
pub mod frame_error;
pub mod frame_event;
pub mod frame_execute;
pub mod frame_options;
pub mod frame_prepare;
pub mod frame_query;
pub mod frame_ready;
pub mod frame_register;
pub mod frame_response;
pub mod frame_result;
pub mod frame_startup;
pub mod frame_supported;
pub mod parser;
pub mod traits;

use crate::error;

static STREAM_ID: AtomicI16 = AtomicI16::new(0);

pub type StreamId = i16;

fn get_next_stream_id() -> StreamId {
    loop {
        let stream = STREAM_ID.fetch_add(1, Ordering::SeqCst);
        if stream < 0 {
            match STREAM_ID.compare_exchange_weak(stream, 0, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => return 0,
                Err(_) => continue,
            }
        }

        return stream;
    }
}

#[derive(Debug)]
pub struct Frame {
    pub version: Version,
    pub flags: Vec<Flag>,
    pub opcode: Opcode,
    pub stream: StreamId,
    pub body: Vec<u8>,
    pub tracing_id: Option<Uuid>,
    pub warnings: Vec<String>,
}

impl Frame {
    pub fn new(
        version: Version,
        flags: Vec<Flag>,
        opcode: Opcode,
        body: Vec<u8>,
        tracing_id: Option<Uuid>,
        warnings: Vec<String>,
    ) -> Self {
        let stream = get_next_stream_id();
        Frame {
            version,
            flags,
            opcode,
            stream,
            body,
            tracing_id,
            warnings,
        }
    }

    pub fn get_body(&self) -> error::Result<ResponseBody> {
        ResponseBody::from(self.body.as_slice(), &self.opcode)
    }

    pub fn tracing_id(&self) -> &Option<Uuid> {
        &self.tracing_id
    }

    pub fn warnings(&self) -> &Vec<String> {
        &self.warnings
    }

    pub fn encode_with(self, compressor: Compression) -> error::Result<Vec<u8>> {
        let mut v = vec![];

        let version_bytes = self.version.as_byte();
        let flag_bytes = Flag::many_to_cbytes(&self.flags);
        let opcode_bytes = self.opcode.as_byte();
        let encoded_body = compressor.encode(self.body)?;
        let body_len = encoded_body.len();

        v.push(version_bytes);
        v.push(flag_bytes);
        v.extend_from_slice(to_n_bytes(self.stream as u64, STREAM_LEN).as_slice());
        v.push(opcode_bytes);
        v.extend_from_slice(to_n_bytes(body_len as u64, LENGTH_LEN).as_slice());
        v.extend_from_slice(encoded_body.as_slice());

        Ok(v)
    }
}

impl AsBytes for Frame {
    fn as_bytes(&self) -> Vec<u8> {
        let mut v = vec![];

        let version_bytes = self.version.as_byte();
        let flag_bytes = Flag::many_to_cbytes(&self.flags);
        let opcode_bytes = self.opcode.as_byte();
        let body_len = self.body.len();

        v.push(version_bytes);
        v.push(flag_bytes);
        v.extend_from_slice(to_n_bytes(self.stream as u64, STREAM_LEN).as_slice());
        v.push(opcode_bytes);
        v.extend_from_slice(to_n_bytes(body_len as u64, LENGTH_LEN).as_slice());
        v.extend_from_slice(self.body.as_slice());

        v
    }
}

/// Frame's version
#[derive(Debug, PartialEq, Copy, Clone, Ord, PartialOrd, Eq, Hash)]
pub enum Version {
    Request,
    Response,
}

impl Version {
    /// Number of bytes that represent Cassandra frame's version.
    pub const BYTE_LENGTH: usize = 1;

    /// It returns an actual Cassandra request frame version that CDRS can work with.
    /// This version is based on selected feature - on of `v3`, `v4` or `v5`.
    fn request_version() -> u8 {
        if cfg!(feature = "v3") {
            0x03
        } else if cfg!(feature = "v4") || cfg!(feature = "v5") {
            0x04
        } else {
            panic!(
                "{}",
                "Protocol version is not supported. CDRS should be run with protocol feature \
                 set to v3, v4 or v5"
            );
        }
    }

    /// It returns an actual Cassandra response frame version that CDRS can work with.
    /// This version is based on selected feature - on of `v3`, `v4` or `v5`.
    fn response_version() -> u8 {
        if cfg!(feature = "v3") {
            0x83
        } else if cfg!(feature = "v4") || cfg!(feature = "v5") {
            0x84
        } else {
            panic!(
                "{}",
                "Protocol version is not supported. CDRS should be run with protocol feature \
                 set to v3, v4 or v5"
            );
        }
    }
}

impl AsByte for Version {
    fn as_byte(&self) -> u8 {
        match self {
            Version::Request => Version::request_version(),
            Version::Response => Version::response_version(),
        }
    }
}

impl From<Vec<u8>> for Version {
    fn from(v: Vec<u8>) -> Version {
        if v.len() != Self::BYTE_LENGTH {
            error!(
                "Unexpected Cassandra verion. Should has {} byte(-s), got {:?}",
                Self::BYTE_LENGTH,
                v
            );
            panic!(
                "Unexpected Cassandra verion. Should has {} byte(-s), got {:?}",
                Self::BYTE_LENGTH,
                v
            );
        }
        let version = v[0];
        let req = Version::request_version();
        let res = Version::response_version();

        if version == req {
            Version::Request
        } else if version == res {
            Version::Response
        } else {
            error!(
                "Unexpected Cassandra version {:?}, either {:?} or {:?} is expected",
                version, req, res
            );
            panic!(
                "Unexpected Cassandra version {:?}, either {:?} or {:?} is expected",
                version, req, res
            );
        }
    }
}

/// Frame's flag
// Is not implemented functionality. Only Igonore works for now
#[derive(Debug, PartialEq)]
pub enum Flag {
    Compression,
    Tracing,
    CustomPayload,
    Warning,
    Ignore,
}

impl Flag {
    /// Number of flag bytes in accordance to protocol.
    const BYTE_LENGTH: usize = 1;

    /// It returns selected flags collection.
    pub fn get_collection(flags: u8) -> Vec<Flag> {
        let mut found_flags: Vec<Flag> = vec![];

        if Flag::has_compression(flags) {
            found_flags.push(Flag::Compression);
        }

        if Flag::has_tracing(flags) {
            found_flags.push(Flag::Tracing);
        }

        if Flag::has_custom_payload(flags) {
            found_flags.push(Flag::CustomPayload);
        }

        if Flag::has_warning(flags) {
            found_flags.push(Flag::Warning);
        }

        found_flags
    }

    /// The method converts a series of `Flag`-s into a single byte.
    pub fn many_to_cbytes(flags: &[Flag]) -> u8 {
        flags
            .iter()
            .fold(Flag::Ignore.as_byte(), |acc, f| acc | f.as_byte())
    }

    /// Indicates if flags contains `Flag::Compression`
    pub fn has_compression(flags: u8) -> bool {
        (flags & Flag::Compression.as_byte()) > 0
    }

    /// Indicates if flags contains `Flag::Tracing`
    pub fn has_tracing(flags: u8) -> bool {
        (flags & Flag::Tracing.as_byte()) > 0
    }

    /// Indicates if flags contains `Flag::CustomPayload`
    pub fn has_custom_payload(flags: u8) -> bool {
        (flags & Flag::CustomPayload.as_byte()) > 0
    }

    /// Indicates if flags contains `Flag::Warning`
    pub fn has_warning(flags: u8) -> bool {
        (flags & Flag::Warning.as_byte()) > 0
    }
}

impl AsByte for Flag {
    fn as_byte(&self) -> u8 {
        match self {
            Flag::Compression => 0x01,
            Flag::Tracing => 0x02,
            Flag::CustomPayload => 0x04,
            Flag::Warning => 0x08,
            Flag::Ignore => 0x00,
            // assuming that ingoing value would be other than [0x01, 0x02, 0x04, 0x08]
        }
    }
}

impl From<u8> for Flag {
    fn from(f: u8) -> Flag {
        match f {
            0x01 => Flag::Compression,
            0x02 => Flag::Tracing,
            0x04 => Flag::CustomPayload,
            0x08 => Flag::Warning,
            _ => Flag::Ignore, // ignore by specification
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Opcode {
    Error,
    Startup,
    Ready,
    Authenticate,
    Options,
    Supported,
    Query,
    Result,
    Prepare,
    Execute,
    Register,
    Event,
    Batch,
    AuthChallenge,
    AuthResponse,
    AuthSuccess,
}

impl Opcode {
    // Number of opcode bytes in accordance to protocol.
    pub const BYTE_LENGTH: usize = 1;
}

impl AsByte for Opcode {
    fn as_byte(&self) -> u8 {
        match self {
            Opcode::Error => 0x00,
            Opcode::Startup => 0x01,
            Opcode::Ready => 0x02,
            Opcode::Authenticate => 0x03,
            Opcode::Options => 0x05,
            Opcode::Supported => 0x06,
            Opcode::Query => 0x07,
            Opcode::Result => 0x08,
            Opcode::Prepare => 0x09,
            Opcode::Execute => 0x0A,
            Opcode::Register => 0x0B,
            Opcode::Event => 0x0C,
            Opcode::Batch => 0x0D,
            Opcode::AuthChallenge => 0x0E,
            Opcode::AuthResponse => 0x0F,
            Opcode::AuthSuccess => 0x10,
        }
    }
}

impl From<u8> for Opcode {
    fn from(b: u8) -> Opcode {
        match b {
            0x00 => Opcode::Error,
            0x01 => Opcode::Startup,
            0x02 => Opcode::Ready,
            0x03 => Opcode::Authenticate,
            0x05 => Opcode::Options,
            0x06 => Opcode::Supported,
            0x07 => Opcode::Query,
            0x08 => Opcode::Result,
            0x09 => Opcode::Prepare,
            0x0A => Opcode::Execute,
            0x0B => Opcode::Register,
            0x0C => Opcode::Event,
            0x0D => Opcode::Batch,
            0x0E => Opcode::AuthChallenge,
            0x0F => Opcode::AuthResponse,
            0x10 => Opcode::AuthSuccess,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::traits::AsByte;

    #[test]
    #[cfg(not(feature = "v3"))]
    fn test_frame_version_as_byte() {
        let request_version = Version::Request;
        assert_eq!(request_version.as_byte(), 0x04);
        let response_version = Version::Response;
        assert_eq!(response_version.as_byte(), 0x84);
    }

    #[test]
    #[cfg(feature = "v3")]
    fn test_frame_version_as_byte_v3() {
        let request_version = Version::Request;
        assert_eq!(request_version.as_byte(), 0x03);
        let response_version = Version::Response;
        assert_eq!(response_version.as_byte(), 0x83);
    }

    #[test]
    #[cfg(not(feature = "v3"))]
    fn test_frame_version_from() {
        let request: Vec<u8> = vec![0x04];
        assert_eq!(Version::from(request), Version::Request);
        let response: Vec<u8> = vec![0x84];
        assert_eq!(Version::from(response), Version::Response);
    }

    #[test]
    #[cfg(feature = "v3")]
    fn test_frame_version_from_v3() {
        let request: Vec<u8> = vec![0x03];
        assert_eq!(Version::from(request), Version::Request);
        let response: Vec<u8> = vec![0x83];
        assert_eq!(Version::from(response), Version::Response);
    }

    #[test]
    fn test_flag_from() {
        assert_eq!(Flag::from(0x01_u8), Flag::Compression);
        assert_eq!(Flag::from(0x02_u8), Flag::Tracing);
        assert_eq!(Flag::from(0x04_u8), Flag::CustomPayload);
        assert_eq!(Flag::from(0x08_u8), Flag::Warning);
        // rest should be interpreted as Ignore
        assert_eq!(Flag::from(0x10_u8), Flag::Ignore);
        assert_eq!(Flag::from(0x31_u8), Flag::Ignore);
    }

    #[test]
    fn test_flag_as_byte() {
        assert_eq!(Flag::Compression.as_byte(), 0x01);
        assert_eq!(Flag::Tracing.as_byte(), 0x02);
        assert_eq!(Flag::CustomPayload.as_byte(), 0x04);
        assert_eq!(Flag::Warning.as_byte(), 0x08);
    }

    #[test]
    fn test_flag_has_x() {
        assert!(Flag::has_compression(0x01));
        assert!(!Flag::has_compression(0x02));

        assert!(Flag::has_tracing(0x02));
        assert!(!Flag::has_tracing(0x01));

        assert!(Flag::has_custom_payload(0x04));
        assert!(!Flag::has_custom_payload(0x02));

        assert!(Flag::has_warning(0x08));
        assert!(!Flag::has_warning(0x01));
    }

    #[test]
    fn test_flag_many_to_cbytes() {
        let all = vec![
            Flag::Compression,
            Flag::Tracing,
            Flag::CustomPayload,
            Flag::Warning,
        ];
        assert_eq!(Flag::many_to_cbytes(&all), 1 | 2 | 4 | 8);
        let some = vec![Flag::Compression, Flag::Warning];
        assert_eq!(Flag::many_to_cbytes(&some), 1 | 8);
        let one = vec![Flag::Compression];
        assert_eq!(Flag::many_to_cbytes(&one), 1);
    }

    #[test]
    fn test_flag_get_collection() {
        let all = vec![
            Flag::Compression,
            Flag::Tracing,
            Flag::CustomPayload,
            Flag::Warning,
        ];
        assert_eq!(Flag::get_collection(1 | 2 | 4 | 8), all);
        let some = vec![Flag::Compression, Flag::Warning];
        assert_eq!(Flag::get_collection(1 | 8), some);
        let one = vec![Flag::Compression];
        assert_eq!(Flag::get_collection(1), one);
    }

    #[test]
    fn test_opcode_as_byte() {
        assert_eq!(Opcode::Error.as_byte(), 0x00);
        assert_eq!(Opcode::Startup.as_byte(), 0x01);
        assert_eq!(Opcode::Ready.as_byte(), 0x02);
        assert_eq!(Opcode::Authenticate.as_byte(), 0x03);
        assert_eq!(Opcode::Options.as_byte(), 0x05);
        assert_eq!(Opcode::Supported.as_byte(), 0x06);
        assert_eq!(Opcode::Query.as_byte(), 0x07);
        assert_eq!(Opcode::Result.as_byte(), 0x08);
        assert_eq!(Opcode::Prepare.as_byte(), 0x09);
        assert_eq!(Opcode::Execute.as_byte(), 0x0A);
        assert_eq!(Opcode::Register.as_byte(), 0x0B);
        assert_eq!(Opcode::Event.as_byte(), 0x0C);
        assert_eq!(Opcode::Batch.as_byte(), 0x0D);
        assert_eq!(Opcode::AuthChallenge.as_byte(), 0x0E);
        assert_eq!(Opcode::AuthResponse.as_byte(), 0x0F);
        assert_eq!(Opcode::AuthSuccess.as_byte(), 0x10);
    }

    #[test]
    fn test_opcode_from() {
        assert_eq!(Opcode::from(0x00), Opcode::Error);
        assert_eq!(Opcode::from(0x01), Opcode::Startup);
        assert_eq!(Opcode::from(0x02), Opcode::Ready);
        assert_eq!(Opcode::from(0x03), Opcode::Authenticate);
        assert_eq!(Opcode::from(0x05), Opcode::Options);
        assert_eq!(Opcode::from(0x06), Opcode::Supported);
        assert_eq!(Opcode::from(0x07), Opcode::Query);
        assert_eq!(Opcode::from(0x08), Opcode::Result);
        assert_eq!(Opcode::from(0x09), Opcode::Prepare);
        assert_eq!(Opcode::from(0x0A), Opcode::Execute);
        assert_eq!(Opcode::from(0x0B), Opcode::Register);
        assert_eq!(Opcode::from(0x0C), Opcode::Event);
        assert_eq!(Opcode::from(0x0D), Opcode::Batch);
        assert_eq!(Opcode::from(0x0E), Opcode::AuthChallenge);
        assert_eq!(Opcode::from(0x0F), Opcode::AuthResponse);
        assert_eq!(Opcode::from(0x10), Opcode::AuthSuccess);
    }
}
