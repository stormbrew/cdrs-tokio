/// Cassandra types
use std::io;
use std::io::{Cursor, Read};
use std::net::SocketAddr;

use crate::error::{column_is_empty_err, Error as CDRSError, Result as CDRSResult};
use crate::frame::traits::{AsBytes, FromBytes, FromCursor};
use crate::types::data_serialization_types::decode_inet;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};

pub const LONG_STR_LEN: usize = 4;
pub const SHORT_LEN: usize = 2;
pub const INT_LEN: usize = 4;
pub const UUID_LEN: usize = 16;

#[macro_use]
pub mod blob;
pub mod data_serialization_types;
pub mod decimal;
pub mod from_cdrs;
pub mod list;
pub mod map;
pub mod rows;
pub mod tuple;
pub mod udt;
pub mod value;

pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::frame::{TryFromRow, TryFromUDT};
    pub use crate::types::blob::Blob;
    pub use crate::types::decimal::Decimal;
    pub use crate::types::list::List;
    pub use crate::types::map::Map;
    pub use crate::types::rows::Row;
    pub use crate::types::tuple::Tuple;
    pub use crate::types::udt::UDT;
    pub use crate::types::value::{Bytes, Value};
    pub use crate::types::AsRustType;
}

/// Should be used to represent a single column as a Rust value.
pub trait AsRustType<T> {
    fn as_rust_type(&self) -> CDRSResult<Option<T>>;

    fn as_r_type(&self) -> CDRSResult<T> {
        self.as_rust_type()
            .and_then(|op| op.ok_or_else(|| CDRSError::from("Value is null or non-set")))
    }
}

pub trait AsRust {
    fn as_rust<R>(&self) -> CDRSResult<Option<R>>
    where
        Self: AsRustType<R>,
    {
        self.as_rust_type()
    }

    fn as_r_rust<T>(&self) -> CDRSResult<T>
    where
        Self: AsRustType<T>,
    {
        self.as_rust()
            .and_then(|op| op.ok_or_else(|| "Value is null or non-set".into()))
    }
}

/// Should be used to return a single column as Rust value by its name.
pub trait IntoRustByName<R> {
    fn get_by_name(&self, name: &str) -> CDRSResult<Option<R>>;

    fn get_r_by_name(&self, name: &str) -> CDRSResult<R> {
        self.get_by_name(name)
            .and_then(|op| op.ok_or_else(|| column_is_empty_err(name)))
    }
}

pub trait ByName {
    fn by_name<R>(&self, name: &str) -> CDRSResult<Option<R>>
    where
        Self: IntoRustByName<R>,
    {
        self.get_by_name(name)
    }

    fn r_by_name<R>(&self, name: &str) -> CDRSResult<R>
    where
        Self: IntoRustByName<R>,
    {
        self.by_name(name)
            .and_then(|op| op.ok_or_else(|| column_is_empty_err(name)))
    }
}

/// Should be used to return a single column as Rust value by its name.
pub trait IntoRustByIndex<R> {
    fn get_by_index(&self, index: usize) -> CDRSResult<Option<R>>;

    fn get_r_by_index(&self, index: usize) -> CDRSResult<R> {
        self.get_by_index(index)
            .and_then(|op| op.ok_or_else(|| column_is_empty_err(index)))
    }
}

pub trait ByIndex {
    fn by_index<R>(&self, index: usize) -> CDRSResult<Option<R>>
    where
        Self: IntoRustByIndex<R>,
    {
        self.get_by_index(index)
    }

    fn r_by_index<R>(&self, index: usize) -> CDRSResult<R>
    where
        Self: IntoRustByIndex<R>,
    {
        self.by_index(index)
            .and_then(|op| op.ok_or_else(|| column_is_empty_err(index)))
    }
}

/// Tries to converts u64 numerical value into array of n bytes.
pub fn try_to_n_bytes(int: u64, n: usize) -> io::Result<Vec<u8>> {
    let mut bytes = vec![];
    bytes.write_uint::<BigEndian>(int, n)?;

    Ok(bytes)
}

/// Converts u64 numerical value into array of n bytes
///
/// # Panics
///
/// It panics if given unsigned integer could not be converted in an array of n bytes
pub fn to_n_bytes(int: u64, n: usize) -> Vec<u8> {
    try_to_n_bytes(int, n).unwrap()
}

pub fn try_i_to_n_bytes(int: i64, n: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(n);
    unsafe {
        bytes.set_len(n);
    }
    BigEndian::write_int(&mut bytes, int, n);

    Ok(bytes)
}

/// Converts u64 numerical value into array of n bytes
///
/// # Panics
///
/// It panics if given integer could not be converted in an array of n bytes
pub fn i_to_n_bytes(int: i64, n: usize) -> Vec<u8> {
    try_i_to_n_bytes(int, n).unwrap()
}

/// Tries to decode bytes array into `u64`.
pub fn try_from_bytes(bytes: &[u8]) -> Result<u64, io::Error> {
    let l = bytes.len();
    let mut c = Cursor::new(bytes);
    c.read_uint::<BigEndian>(l)
}

/// Tries to decode bytes array into `u16`.
pub fn try_u16_from_bytes(bytes: &[u8]) -> Result<u16, io::Error> {
    let mut c = Cursor::new(bytes);
    c.read_u16::<BigEndian>()
}

/// Tries to decode bytes array into `i64`.
pub fn try_i_from_bytes(bytes: &[u8]) -> Result<i64, io::Error> {
    let l = bytes.len();
    let mut c = Cursor::new(bytes);
    c.read_int::<BigEndian>(l)
}

/// Tries to decode bytes array into `i32`.
pub fn try_i32_from_bytes(bytes: &[u8]) -> Result<i32, io::Error> {
    let mut c = Cursor::new(bytes);
    c.read_i32::<BigEndian>()
}

/// Tries to decode bytes array into `i16`.
pub fn try_i16_from_bytes(bytes: &[u8]) -> Result<i16, io::Error> {
    let mut c = Cursor::new(bytes);
    c.read_i16::<BigEndian>()
}

/// Tries to decode bytes array into `f32`.
pub fn try_f32_from_bytes(bytes: &[u8]) -> Result<f32, io::Error> {
    let mut c = Cursor::new(bytes);
    c.read_f32::<BigEndian>()
}

/// Tries to decode bytes array into `f64`.
pub fn try_f64_from_bytes(bytes: &[u8]) -> Result<f64, io::Error> {
    let mut c = Cursor::new(bytes);
    c.read_f64::<BigEndian>()
}

/// Converts byte-array into u64
///
/// # Panics
///
/// It panics if given bytes could not be converted into `u64`
pub fn from_bytes(bytes: &[u8]) -> u64 {
    try_from_bytes(bytes).unwrap()
}

/// Converts byte-array into i64
///
/// # Panics
///
/// It panics if given bytes could not be converted into `i64`
pub fn from_i_bytes(bytes: &[u8]) -> i64 {
    try_i_from_bytes(bytes).unwrap()
}

/// Converts byte-array into u16
///
/// # Panics
///
/// It panics if given bytes could not be converted into `u16`
pub fn from_u16_bytes(bytes: &[u8]) -> u16 {
    try_u16_from_bytes(bytes).unwrap()
}

/// Converts byte-array into i16
///
/// # Panics
///
/// It panics if given bytes could not be converted into `u16`
pub fn from_i16_bytes(bytes: &[u8]) -> i16 {
    try_i16_from_bytes(bytes).unwrap()
}

/// Converts number i16 into Cassandra's short.
///
/// # Panics
///
/// It panics if given `i16` could not be converted into bytes
pub fn to_short(int: i16) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is i16
    let _ = bytes.write_i16::<BigEndian>(int).unwrap();

    bytes
}

/// Converts integer into Cassandra's int
///
/// # Panics
///
/// It panics if given `i32` could not be converted into bytes
pub fn to_int(int: i32) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is i16
    let _ = bytes.write_i32::<BigEndian>(int).unwrap();

    bytes
}

/// Converts integer into Cassandra's int
///
/// # Panics
///
/// It panics if given `i64` could not be converted into bytes
pub fn to_bigint(int: i64) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is i64
    let _ = bytes.write_i64::<BigEndian>(int).unwrap();

    bytes
}

/// Converts integer into Cassandra's varint.
pub fn to_varint(int: i64) -> Vec<u8> {
    if int == 0 {
        return vec![0];
    }

    let mut int_bytes = to_bigint(int);
    match int.signum() {
        1 => {
            int_bytes = int_bytes.into_iter().skip_while(|b| *b == 0x00).collect();
            if int_bytes
                .get(0)
                .map(|b| b.leading_zeros() == 0)
                .unwrap_or(true)
            {
                int_bytes.insert(0, 0x00);
            }
        }
        -1 => {
            int_bytes = int_bytes.into_iter().skip_while(|b| *b == 0xFF).collect();
            if int_bytes
                .get(0)
                .map(|b| b.leading_zeros() > 0)
                .unwrap_or(true)
            {
                int_bytes.insert(0, 0xFF);
            }
        }
        _ => unreachable!(),
    }

    int_bytes
}

/// Converts number i16 into Cassandra's `short`.
///
/// # Panics
///
/// It panics if given `u16` could not be converted into bytes
pub fn to_u_short(int: u16) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is i16
    let _ = bytes.write_u16::<BigEndian>(int).unwrap();

    bytes
}

/// Converts integer into Cassandra's int
///
/// # Panics
///
/// It panics if given `u32` could not be converted into bytes
pub fn to_u(int: u32) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is u64
    let _ = bytes.write_u32::<BigEndian>(int).unwrap();

    bytes
}

/// Converts integer into Cassandra's `int`
///
/// # Panics
///
/// It panics if given `u64` could not be converted into `u64`
pub fn to_u_big(int: u64) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is u64
    let _ = bytes.write_u64::<BigEndian>(int).unwrap();

    bytes
}

/// Converts `f32` into bytes
///
/// # Panics
///
/// It panics if given `f32` could not be converted into bytes
pub fn to_float(f: f32) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is f32
    let _ = bytes.write_f32::<BigEndian>(f).unwrap();

    bytes
}

/// Converts `f64` into array of bytes
///
/// # Panics
///
/// It panics if given `f63` could not be converted into bytes
pub fn to_float_big(f: f64) -> Vec<u8> {
    let mut bytes = vec![];
    // should not panic as input is f64
    let _ = bytes.write_f64::<BigEndian>(f).unwrap();

    bytes
}

#[derive(Debug, Clone)]
pub struct CString {
    string: String,
}

impl CString {
    pub fn new(string: String) -> CString {
        CString { string }
    }

    /// Converts internal value into pointer of `str`.
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    /// Converts internal value into a plain `String`.
    pub fn into_plain(self) -> String {
        self.string
    }

    /// Represents internal value as a `String`.
    pub fn as_plain(&self) -> String {
        self.string.clone()
    }
}

// Implementation for Rust std types
// Use extended Rust string as Cassandra [string]
impl AsBytes for CString {
    /// Converts into Cassandra byte representation of string
    fn as_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = vec![];
        let l = self.string.len() as i16;
        v.extend_from_slice(to_short(l).as_slice());
        v.extend_from_slice(self.string.as_bytes());
        v
    }
}

impl FromCursor for CString {
    /// from_cursor gets Cursor who's position is set such that it should be a start of a string.
    /// It reads required number of bytes and returns a String
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CString> {
        let mut buff = [0; SHORT_LEN];
        let len_bytes = cursor_fill_value(&mut cursor, &mut buff)?;
        let len: u64 = try_from_bytes(len_bytes)?;
        let body_bytes = cursor_next_value(&mut cursor, len)?;

        String::from_utf8(body_bytes)
            .map_err(Into::into)
            .map(CString::new)
    }
}

#[derive(Debug, Clone)]
pub struct CStringLong {
    string: String,
}

impl CStringLong {
    pub fn new(string: String) -> CStringLong {
        CStringLong { string }
    }

    /// Converts internal value into pointer of `str`.
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    /// Converts internal value into a plain `String`.
    pub fn into_plain(self) -> String {
        self.string
    }
}

// Implementation for Rust std types
// Use extended Rust string as Cassandra [string]
impl AsBytes for CStringLong {
    /// Converts into Cassandra byte representation of string
    fn as_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = vec![];
        let l = self.string.len() as i32;
        v.extend_from_slice(to_int(l).as_slice());
        v.extend_from_slice(self.string.as_bytes());
        v
    }
}

impl FromCursor for CStringLong {
    /// from_cursor gets Cursor who's position is set such that it should be a start of a string.
    /// It reads required number of bytes and returns a String
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CStringLong> {
        let mut buff = [0; INT_LEN];
        let len_bytes = cursor_fill_value(&mut cursor, &mut buff)?;
        let len: u64 = try_from_bytes(len_bytes)?;
        let body_bytes = cursor_next_value(&mut cursor, len)?;

        String::from_utf8(body_bytes)
            .map_err(Into::into)
            .map(CStringLong::new)
    }
}

#[derive(Debug, Clone)]
pub struct CStringList {
    pub list: Vec<CString>,
}

impl CStringList {
    pub fn into_plain(self) -> Vec<String> {
        self.list
            .iter()
            .map(|string| string.clone().into_plain())
            .collect()
    }
}

impl AsBytes for CStringList {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let l = to_short(self.list.len() as i16);
        bytes.extend_from_slice(l.as_slice());

        bytes = self.list.iter().fold(bytes, |mut _bytes, cstring| {
            _bytes.extend_from_slice(cstring.as_bytes().as_slice());
            _bytes
        });

        bytes
    }
}

impl FromCursor for CStringList {
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CStringList> {
        // TODO: try to use slice instead
        let mut len_bytes = [0; SHORT_LEN];
        cursor.read_exact(&mut len_bytes)?;
        let len = try_from_bytes(len_bytes.to_vec().as_slice())? as usize;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(CString::from_cursor(&mut cursor)?);
        }

        Ok(CStringList { list })
    }
}

//

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// The structure that represents Cassandra byte type.
pub struct CBytes {
    bytes: Option<Vec<u8>>,
}

impl CBytes {
    pub fn new(bytes: Vec<u8>) -> CBytes {
        CBytes { bytes: Some(bytes) }
    }

    /// Creates Cassandra bytes that represent empty or null value
    pub fn new_empty() -> CBytes {
        CBytes { bytes: None }
    }

    /// Converts `CBytes` into a plain array of bytes
    pub fn into_plain(self) -> Option<Vec<u8>> {
        self.bytes
    }

    // TODO: try to replace usage of `as_plain` by `as_slice`
    pub fn as_plain(&self) -> Option<Vec<u8>> {
        self.bytes.clone()
    }
    pub fn as_slice(&self) -> Option<&[u8]> {
        match self.bytes {
            Some(ref v) => Some(v.as_slice()),
            None => None,
        }
        // self.bytes.map(|v| v.as_slice())
    }
    pub fn is_empty(&self) -> bool {
        match &self.bytes {
            None => true,
            Some(bytes) => bytes.is_empty(),
        }
    }
}

impl FromCursor for CBytes {
    /// from_cursor gets Cursor who's position is set such that it should be a start of bytes.
    /// It reads required number of bytes and returns a CBytes
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CBytes> {
        let len = CInt::from_cursor(&mut cursor)?;
        // null or not set value
        if len < 0 {
            return Ok(CBytes { bytes: None });
        }

        cursor_next_value(&mut cursor, len as u64).map(CBytes::new)
    }
}

// Use extended Rust Vec<u8> as Cassandra [bytes]
impl AsBytes for CBytes {
    fn as_bytes(&self) -> Vec<u8> {
        match self.bytes {
            Some(ref b) => {
                let mut v: Vec<u8> = vec![];
                let l = b.len() as i32;
                v.extend_from_slice(to_int(l).as_slice());
                v.extend_from_slice(b.as_slice());
                v
            }
            None => vec![],
        }
    }
}

/// Cassandra short bytes
#[derive(Debug, Clone)]
pub struct CBytesShort {
    bytes: Option<Vec<u8>>,
}

impl CBytesShort {
    pub fn new(bytes: Vec<u8>) -> CBytesShort {
        CBytesShort { bytes: Some(bytes) }
    }
    /// Converts `CBytesShort` into plain vector of bytes;
    pub fn into_plain(self) -> Option<Vec<u8>> {
        self.bytes
    }
}

impl FromCursor for CBytesShort {
    /// from_cursor gets Cursor who's position is set such that it should be a start of bytes.
    /// It reads required number of bytes and returns a CBytes
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CBytesShort> {
        let len = CIntShort::from_cursor(&mut cursor)?;

        if len < 0 {
            return Ok(CBytesShort { bytes: None });
        }

        cursor_next_value(&mut cursor, len as u64)
            .map(CBytesShort::new)
            .map_err(Into::into)
    }
}

// Use extended Rust Vec<u8> as Cassandra [bytes]
impl AsBytes for CBytesShort {
    fn as_bytes(&self) -> Vec<u8> {
        match self.bytes {
            Some(ref b) => {
                let mut v: Vec<u8> = vec![];
                let l = b.len() as i16;
                v.extend_from_slice(to_short(l).as_slice());
                v.extend_from_slice(b.as_slice());
                v
            }
            None => vec![],
        }
    }
}

/// Cassandra int type.
pub type CInt = i32;

impl FromCursor for CInt {
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CInt> {
        let mut buff = [0; INT_LEN];
        let bytes = cursor_fill_value(&mut cursor, &mut buff)?;
        try_i32_from_bytes(bytes).map_err(Into::into)
    }
}

/// Cassandra int short type.
pub type CIntShort = i16;

impl FromCursor for CIntShort {
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CIntShort> {
        let mut buff = [0; SHORT_LEN];
        let bytes = cursor_fill_value(&mut cursor, &mut buff)?;
        try_i16_from_bytes(bytes).map_err(Into::into)
    }
}

// Use extended Rust Vec<u8> as Cassandra [bytes]
impl FromBytes for Vec<u8> {
    fn from_bytes(bytes: &[u8]) -> CDRSResult<Vec<u8>> {
        let mut cursor = Cursor::new(bytes);
        let mut buff = [0; SHORT_LEN];
        let len_bytes = cursor_fill_value(&mut cursor, &mut buff)?;
        let len: u64 = try_from_bytes(len_bytes)?;

        cursor_next_value(&mut cursor, len).map_err(Into::into)
    }
}

/// The structure which represents Cassandra inet
/// (https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L222).
#[derive(Debug)]
pub struct CInet {
    pub addr: SocketAddr,
}

impl FromCursor for CInet {
    fn from_cursor(mut cursor: &mut Cursor<&[u8]>) -> CDRSResult<CInet> {
        let n = cursor_fill_value(&mut cursor, &mut [0])?[0];
        let ip = decode_inet(cursor_next_value(&mut cursor, n as u64)?.as_slice())?;
        let port = CInt::from_cursor(&mut cursor)?;
        let socket_addr = SocketAddr::new(ip, port as u16);

        Ok(CInet { addr: socket_addr })
    }
}

pub fn cursor_next_value(cursor: &mut Cursor<&[u8]>, len: u64) -> CDRSResult<Vec<u8>> {
    let l = len as usize;
    let current_position = cursor.position();
    let mut buff: Vec<u8> = Vec::with_capacity(l);
    unsafe {
        buff.set_len(l);
    }
    cursor.read_exact(&mut buff)?;
    cursor.set_position(current_position + len);
    Ok(buff)
}

pub fn cursor_fill_value<'a>(
    cursor: &mut Cursor<&[u8]>,
    buff: &'a mut [u8],
) -> CDRSResult<&'a [u8]> {
    let current_position = cursor.position();
    cursor.read_exact(buff)?;
    cursor.set_position(current_position + buff.len() as u64);
    Ok(buff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::traits::{AsBytes, FromCursor};
    use std::io::Cursor;
    use std::mem::transmute;

    // CString
    #[test]
    fn test_cstring_new() {
        let value = "foo".to_string();
        let _ = CString::new(value);
    }

    #[test]
    fn test_cstring_as_str() {
        let value = "foo".to_string();
        let cstring = CString::new(value);

        assert_eq!(cstring.as_str(), "foo");
    }

    #[test]
    fn test_cstring_into_plain() {
        let value = "foo".to_string();
        let cstring = CString::new(value);

        assert_eq!(cstring.into_plain(), "foo".to_string());
    }

    #[test]
    fn test_cstring_into_cbytes() {
        let value = "foo".to_string();
        let cstring = CString::new(value);

        assert_eq!(cstring.as_bytes(), &[0, 3, 102, 111, 111]);
    }

    #[test]
    fn test_cstring_from_cursor() {
        let a = &[0, 3, 102, 111, 111, 0];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let cstring = CString::from_cursor(&mut cursor).unwrap();
        assert_eq!(cstring.as_str(), "foo");
    }

    // CStringLong
    #[test]
    fn test_cstringlong_new() {
        let value = "foo".to_string();
        let _ = CStringLong::new(value);
    }

    #[test]
    fn test_cstringlong_as_str() {
        let value = "foo".to_string();
        let cstring = CStringLong::new(value);

        assert_eq!(cstring.as_str(), "foo");
    }

    #[test]
    fn test_cstringlong_into_plain() {
        let value = "foo".to_string();
        let cstring = CStringLong::new(value);

        assert_eq!(cstring.into_plain(), "foo".to_string());
    }

    #[test]
    fn test_cstringlong_into_cbytes() {
        let value = "foo".to_string();
        let cstring = CStringLong::new(value);

        assert_eq!(cstring.as_bytes(), &[0, 0, 0, 3, 102, 111, 111]);
    }

    #[test]
    fn test_cstringlong_from_cursor() {
        let a = &[0, 0, 0, 3, 102, 111, 111, 0];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let cstring = CStringLong::from_cursor(&mut cursor).unwrap();
        assert_eq!(cstring.as_str(), "foo");
    }

    // CStringList
    #[test]
    fn test_cstringlist() {
        let a = &[0, 2, 0, 3, 102, 111, 111, 0, 3, 102, 111, 111];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let list = CStringList::from_cursor(&mut cursor).unwrap();
        let plain = list.into_plain();
        assert_eq!(plain.len(), 2);
        for s in plain.iter() {
            assert_eq!(s.as_str(), "foo");
        }
    }

    // CBytes
    #[test]
    fn test_cbytes_new() {
        let bytes_vec = vec![1, 2, 3];
        let _ = CBytes::new(bytes_vec);
    }

    #[test]
    fn test_cbytes_into_plain() {
        let cbytes = CBytes::new(vec![1, 2, 3]);
        assert_eq!(cbytes.into_plain().unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn test_cbytes_from_cursor() {
        let a = &[0, 0, 0, 3, 1, 2, 3];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let cbytes = CBytes::from_cursor(&mut cursor).unwrap();
        assert_eq!(cbytes.into_plain().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_cbytes_into_cbytes() {
        let bytes_vec = vec![1, 2, 3];
        let cbytes = CBytes::new(bytes_vec);
        assert_eq!(cbytes.as_bytes(), vec![0, 0, 0, 3, 1, 2, 3]);
    }

    // CBytesShort
    #[test]
    fn test_cbytesshort_new() {
        let bytes_vec = vec![1, 2, 3];
        let _ = CBytesShort::new(bytes_vec);
    }

    #[test]
    fn test_cbytesshort_into_plain() {
        let cbytes = CBytesShort::new(vec![1, 2, 3]);
        assert_eq!(cbytes.into_plain().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_cbytesshort_from_cursor() {
        let a = &[0, 3, 1, 2, 3];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let cbytes = CBytesShort::from_cursor(&mut cursor).unwrap();
        assert_eq!(cbytes.into_plain().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_cbytesshort_into_cbytes() {
        let bytes_vec: Vec<u8> = vec![1, 2, 3];
        let cbytes = CBytesShort::new(bytes_vec);
        assert_eq!(cbytes.as_bytes(), vec![0, 3, 1, 2, 3]);
    }

    // CInt
    #[test]
    fn test_cint_from_cursor() {
        let a = &[0, 0, 0, 5];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let i = CInt::from_cursor(&mut cursor).unwrap();
        assert_eq!(i, 5);
    }

    // CIntShort
    #[test]
    fn test_cintshort_from_cursor() {
        let a = &[0, 5];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let i = CIntShort::from_cursor(&mut cursor).unwrap();
        assert_eq!(i, 5);
    }

    // cursor_next_value
    #[test]
    fn test_cursor_next_value() {
        let a = &[0, 1, 2, 3, 4];
        let mut cursor: Cursor<&[u8]> = Cursor::new(a);
        let l: u64 = 3;
        let val = cursor_next_value(&mut cursor, l).unwrap();
        assert_eq!(val, vec![0, 1, 2]);
    }

    #[test]
    fn test_try_u16_from_bytes() {
        let bytes: [u8; 2] = unsafe { transmute(12u16.to_be()) }; // or .to_le()
        let val = try_u16_from_bytes(&bytes);
        assert_eq!(val.unwrap(), 12u16);
    }

    #[test]
    fn test_from_i_bytes() {
        let bytes: [u8; 8] = unsafe { transmute(12i64.to_be()) }; // or .to_le()
        let val = from_i_bytes(&bytes);
        assert_eq!(val, 12i64);
    }

    #[test]
    fn test_to_varint() {
        assert_eq!(to_varint(0), vec![0x00]);
        assert_eq!(to_varint(1), vec![0x01]);
        assert_eq!(to_varint(127), vec![0x7F]);
        assert_eq!(to_varint(128), vec![0x00, 0x80]);
        assert_eq!(to_varint(129), vec![0x00, 0x81]);
        assert_eq!(to_varint(-1), vec![0xFF]);
        assert_eq!(to_varint(-128), vec![0x80]);
        assert_eq!(to_varint(-129), vec![0xFF, 0x7F]);
    }
}
