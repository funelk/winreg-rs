// Copyright 2023, Igor Shaula
// Licensed under the MIT License <LICENSE or
// http://opensource.org/licenses/MIT>. This file
// may not be copied, modified, or distributed
// except according to those terms.
use self::EncoderState::*;
use crate::enums::*;
use crate::reg_key::RegKey;
use crate::transaction::Transaction;
use std::error::Error;
use std::fmt;
use std::io;

macro_rules! emit_value {
    ($s:ident, $v:ident) => {
        match mem::replace(&mut $s.state, Start) {
            NextKey(ref s) => $s.keys[$s.keys.len() - 1]
                .set_value(s, &$v)
                .map_err(EncoderError::IoError),
            Start => Err(EncoderError::NoFieldName),
        }
    };
}

macro_rules! no_impl {
    ($e:expr) => {
        Err(EncoderError::EncodeNotImplemented($e.to_owned()))
    };
}

#[cfg(feature = "serialization-serde")]
mod serialization_serde;

#[derive(Debug)]
pub enum EncoderError {
    EncodeNotImplemented(String),
    SerializerError(String),
    IoError(io::Error),
    NoFieldName,
    KeyMustBeAString,
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for EncoderError {}

pub type EncodeResult<T> = Result<T, EncoderError>;

impl From<io::Error> for EncoderError {
    fn from(err: io::Error) -> EncoderError {
        EncoderError::IoError(err)
    }
}

#[derive(Debug)]
enum EncoderState {
    Start,
    NextKey(String),
    // NextMapKey,
}

#[derive(Debug)]
pub struct Encoder<Tr: AsRef<Transaction>> {
    keys: Vec<RegKey>,
    tr: Tr,
    state: EncoderState,
}

const ENCODER_SAM: u32 = KEY_CREATE_SUB_KEY | KEY_SET_VALUE;

impl Encoder<Transaction> {
    pub fn from_key(key: &RegKey) -> EncodeResult<Encoder<Transaction>> {
        let tr = Transaction::new()?;
        key.open_subkey_transacted_with_flags("", &tr, ENCODER_SAM)
            .map(|k| Encoder::new(k, tr))
            .map_err(EncoderError::IoError)
    }

    fn new(key: RegKey, tr: Transaction) -> Encoder<Transaction> {
        let mut keys = Vec::with_capacity(5);
        keys.push(key);
        Encoder {
            keys,
            tr,
            state: Start,
        }
    }

    pub fn commit(self) -> EncodeResult<()> {
        self.tr.commit().map_err(EncoderError::IoError)
    }
}

impl Encoder<&Transaction> {
    pub fn from_key_transacted<'a>(
        key: &RegKey,
        tr: &'a Transaction,
    ) -> EncodeResult<Encoder<&'a Transaction>> {
        key.open_subkey_transacted_with_flags("", tr, ENCODER_SAM)
            .map(|k| Encoder::new_transacted(k, tr))
            .map_err(EncoderError::IoError)
    }

    fn new_transacted(key: RegKey, tr: &Transaction) -> Encoder<&Transaction> {
        let mut keys = Vec::with_capacity(5);
        keys.push(key);
        Encoder {
            keys,
            tr,
            state: Start,
        }
    }
}
