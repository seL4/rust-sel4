//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::error::Error;
use core::fmt;
use core::marker::PhantomData;
use core::mem;

use serde::{Deserialize, Serialize};
use zerocopy::IntoBytes;

use crate::{bytes_to_words, MessageLabel, MessageReader, MessageRegisterValue, MessageWriter};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PostcardMessageReader;

impl PostcardMessageReader {
    pub const fn new() -> Self {
        Self
    }
}

impl<T: for<'a> Deserialize<'a>> MessageReader<T> for PostcardMessageReader {
    type Error = PostcardMessageReaderError;

    fn read_message(
        &self,
        label: MessageLabel,
        buf: &[MessageRegisterValue],
    ) -> Result<T, Self::Error> {
        if label != 0 {
            return Err(PostcardMessageReaderError::UnexpectedLabel { label });
        }
        postcard::from_bytes(buf.as_bytes())
            .map_err(PostcardMessageReaderError::DeserializationError)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PostcardMessageWriter<T>(T);

impl<T> PostcardMessageWriter<T> {
    pub const fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T: Serialize> MessageWriter for PostcardMessageWriter<T> {
    type Error = postcard::Error;

    fn write_message(
        &self,
        buf: &mut [MessageRegisterValue],
    ) -> Result<(MessageLabel, usize), Self::Error> {
        postcard::to_slice(&self.0, buf.as_mut_bytes()).map(|used| (0, bytes_to_words(used.len())))
    }
}

#[derive(Clone, Debug)]
pub enum PostcardMessageReaderError {
    UnexpectedLabel { label: MessageLabel },
    DeserializationError(postcard::Error),
}

impl fmt::Display for PostcardMessageReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedLabel { label } => write!(f, "unexpected label: {label}"),
            Self::DeserializationError(err) => write!(f, "deserialization error: {err}"),
        }
    }
}

impl Error for PostcardMessageReaderError {}
