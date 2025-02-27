use std::io::Cursor;

use crate::error;
use crate::frame::frame_auth_challenge::*;
use crate::frame::frame_auth_success::BodyReqAuthSuccess;
use crate::frame::frame_authenticate::BodyResAuthenticate;
use crate::frame::frame_error::CDRSError;
use crate::frame::frame_event::BodyResEvent;
use crate::frame::frame_result::{
    BodyResResultPrepared, BodyResResultRows, BodyResResultSetKeyspace, BodyResResultVoid,
    ResResultBody, RowsMetadata,
};
use crate::frame::frame_supported::*;
use crate::frame::FromCursor;
use crate::frame::Opcode;
use crate::types::rows::Row;

#[derive(Debug)]
pub enum ResponseBody {
    Error(CDRSError),
    Startup,
    Ready(BodyResResultVoid),
    Authenticate(BodyResAuthenticate),
    Options,
    Supported(BodyResSupported),
    Query,
    Result(ResResultBody),
    Prepare,
    Execute,
    Register,
    Event(BodyResEvent),
    Batch,
    AuthChallenge(BodyResAuthChallenge),
    AuthResponse,
    AuthSuccess(BodyReqAuthSuccess),
}

impl ResponseBody {
    pub fn from(bytes: &[u8], response_type: &Opcode) -> error::Result<ResponseBody> {
        let mut cursor: Cursor<&[u8]> = Cursor::new(bytes);
        Ok(match *response_type {
            // request frames
            Opcode::Startup => unreachable!(),
            Opcode::Options => unreachable!(),
            Opcode::Query => unreachable!(),
            Opcode::Prepare => unreachable!(),
            Opcode::Execute => unreachable!(),
            Opcode::Register => unreachable!(),
            Opcode::Batch => unreachable!(),
            Opcode::AuthResponse => unreachable!(),

            // response frames
            Opcode::Error => ResponseBody::Error(CDRSError::from_cursor(&mut cursor)?),
            Opcode::Ready => ResponseBody::Ready(BodyResResultVoid::from_cursor(&mut cursor)?),
            Opcode::Authenticate => {
                ResponseBody::Authenticate(BodyResAuthenticate::from_cursor(&mut cursor)?)
            }
            Opcode::Supported => {
                ResponseBody::Supported(BodyResSupported::from_cursor(&mut cursor)?)
            }
            Opcode::Result => ResponseBody::Result(ResResultBody::from_cursor(&mut cursor)?),
            Opcode::Event => ResponseBody::Event(BodyResEvent::from_cursor(&mut cursor)?),
            Opcode::AuthChallenge => {
                ResponseBody::AuthChallenge(BodyResAuthChallenge::from_cursor(&mut cursor)?)
            }
            Opcode::AuthSuccess => {
                ResponseBody::AuthSuccess(BodyReqAuthSuccess::from_cursor(&mut cursor)?)
            }
        })
    }

    pub fn into_rows(self) -> Option<Vec<Row>> {
        match self {
            ResponseBody::Result(res) => res.into_rows(),
            _ => None,
        }
    }

    pub fn as_rows_metadata(&self) -> Option<RowsMetadata> {
        match *self {
            ResponseBody::Result(ref res) => res.as_rows_metadata(),
            _ => None,
        }
    }

    pub fn as_cols(&self) -> Option<&BodyResResultRows> {
        match *self {
            ResponseBody::Result(ResResultBody::Rows(ref rows)) => Some(rows),
            _ => None,
        }
    }

    /// It unwraps body and returns BodyResResultPrepared which contains an exact result of
    /// PREPARE query. If frame body is not of type `Result` this method returns `None`.
    pub fn into_prepared(self) -> Option<BodyResResultPrepared> {
        match self {
            ResponseBody::Result(res) => res.into_prepared(),
            _ => None,
        }
    }

    /// It unwraps body and returns BodyResResultPrepared which contains an exact result of
    /// use keyspace query. If frame body is not of type `Result` this method returns `None`.
    pub fn into_set_keyspace(self) -> Option<BodyResResultSetKeyspace> {
        match self {
            ResponseBody::Result(res) => res.into_set_keyspace(),
            _ => None,
        }
    }

    /// It unwraps body and returns BodyResEvent.
    /// If frame body is not of type `Result` this method returns `None`.
    pub fn into_server_event(self) -> Option<BodyResEvent> {
        match self {
            ResponseBody::Event(event) => Some(event),
            _ => None,
        }
    }

    pub fn get_authenticator(&self) -> Option<&str> {
        match *self {
            ResponseBody::Authenticate(ref auth) => Some(auth.data.as_str()),
            _ => None,
        }
    }
}
