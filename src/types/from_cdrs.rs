use std::net::IpAddr;
use std::num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8};

use chrono::prelude::*;
use time::PrimitiveDateTime;
use uuid::Uuid;

use crate::error::Result as CDRSResult;
use crate::types::blob::Blob;
use crate::types::decimal::Decimal;
use crate::types::list::List;
use crate::types::map::Map;
use crate::types::tuple::Tuple;
use crate::types::udt::UDT;
use crate::types::{AsRustType, ByName, IntoRustByName};

pub trait FromCDRS {
    fn from_cdrs<T>(cdrs_type: T) -> CDRSResult<Option<Self>>
    where
        Self: Sized,
        T: AsRustType<Self>,
    {
        cdrs_type.as_rust_type()
    }

    fn from_cdrs_r<T>(cdrs_type: T) -> CDRSResult<Self>
    where
        Self: Sized,
        T: AsRustType<Self>,
    {
        cdrs_type.as_r_type()
    }
}

impl FromCDRS for Blob {}
impl FromCDRS for String {}
impl FromCDRS for bool {}
impl FromCDRS for i64 {}
impl FromCDRS for i32 {}
impl FromCDRS for i16 {}
impl FromCDRS for i8 {}
impl FromCDRS for f64 {}
impl FromCDRS for f32 {}
impl FromCDRS for IpAddr {}
impl FromCDRS for Uuid {}
impl FromCDRS for List {}
impl FromCDRS for Map {}
impl FromCDRS for UDT {}
impl FromCDRS for Tuple {}
impl FromCDRS for PrimitiveDateTime {}
impl FromCDRS for Decimal {}
impl FromCDRS for NonZeroI8 {}
impl FromCDRS for NonZeroI16 {}
impl FromCDRS for NonZeroI32 {}
impl FromCDRS for NonZeroI64 {}
impl FromCDRS for NaiveDateTime {}
impl<Tz: TimeZone> FromCDRS for DateTime<Tz> {}

pub trait FromCDRSByName {
    fn from_cdrs_by_name<T>(cdrs_type: &T, name: &str) -> CDRSResult<Option<Self>>
    where
        Self: Sized,
        T: ByName + IntoRustByName<Self>,
    {
        cdrs_type.by_name(name)
    }

    fn from_cdrs_r<T>(cdrs_type: &T, name: &str) -> CDRSResult<Self>
    where
        Self: Sized,
        T: ByName + IntoRustByName<Self> + ::std::fmt::Debug,
    {
        cdrs_type.r_by_name(name)
    }
}

impl FromCDRSByName for Blob {}
impl FromCDRSByName for String {}
impl FromCDRSByName for bool {}
impl FromCDRSByName for i64 {}
impl FromCDRSByName for i32 {}
impl FromCDRSByName for i16 {}
impl FromCDRSByName for i8 {}
impl FromCDRSByName for f64 {}
impl FromCDRSByName for f32 {}
impl FromCDRSByName for IpAddr {}
impl FromCDRSByName for Uuid {}
impl FromCDRSByName for List {}
impl FromCDRSByName for Map {}
impl FromCDRSByName for UDT {}
impl FromCDRSByName for Tuple {}
impl FromCDRSByName for PrimitiveDateTime {}
impl FromCDRSByName for Decimal {}
impl FromCDRSByName for NonZeroI8 {}
impl FromCDRSByName for NonZeroI16 {}
impl FromCDRSByName for NonZeroI32 {}
impl FromCDRSByName for NonZeroI64 {}
impl FromCDRSByName for NaiveDateTime {}
impl<Tz: TimeZone> FromCDRSByName for DateTime<Tz> {}
