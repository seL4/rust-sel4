//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::fmt;
use core::marker::PhantomData;
use core::mem;

use serde::{Deserialize, Serialize};

use sel4::{Badge, Endpoint, IpcBuffer, MessageInfo, MessageInfoBuilder, Word};

const BYTES_PER_WORD: usize = mem::size_of::<Word>() / mem::size_of::<u8>();

#[derive(Clone)]
pub struct Client<T> {
    endpoint: Endpoint,
    phantom: PhantomData<T>,
}

impl<T: Serialize> Client<T> {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            phantom: PhantomData,
        }
    }

    pub fn send(&self, data: &T) -> Result<(), Error> {
        let info = prepare_data_for_send(data)?;
        self.endpoint.send(info);
        Ok(())
    }

    pub fn call<U: for<'a> Deserialize<'a>>(&self, data: &T) -> Result<U, Error> {
        let send_info = prepare_data_for_send(data)?;
        let recv_info = self.endpoint.call(send_info);
        recv_data(&recv_info)
    }
}

pub mod server {
    use super::*;

    pub fn recv<T, F>(endpoint: Endpoint, f: F) -> T
    where
        F: FnOnce(Reception) -> T,
    {
        let (info, badge) = endpoint.recv(());
        sel4::with_ipc_buffer(|ipc_buffer| {
            let reception = Reception::new(info, badge, ipc_buffer);
            f(reception)
        })
    }

    pub fn send<T: Serialize>(endpoint: Endpoint, data: &T) -> Result<(), Error> {
        let info = prepare_data_for_send(data)?;
        endpoint.send(info);
        Ok(())
    }

    pub fn reply<T: Serialize>(data: &T) -> Result<(), Error> {
        let info = prepare_data_for_send(data)?;
        sel4::with_ipc_buffer_mut(|ipc_buffer| sel4::reply(ipc_buffer, info));
        Ok(())
    }
}

pub struct Reception<'a> {
    info: MessageInfo,
    badge: Badge,
    ipc_buffer: &'a IpcBuffer,
}

impl<'a> Reception<'a> {
    fn new(info: MessageInfo, badge: Badge, ipc_buffer: &'a IpcBuffer) -> Self {
        Self {
            info,
            badge,
            ipc_buffer,
        }
    }

    pub fn info(&self) -> &MessageInfo {
        &self.info
    }

    pub fn badge(&self) -> Badge {
        self.badge
    }

    pub fn ipc_buffer(&self) -> &IpcBuffer {
        self.ipc_buffer
    }

    pub fn read<T: for<'b> Deserialize<'b>>(&self) -> Result<T, Error> {
        recv_data_with_ipc_buffer(&self.info, self.ipc_buffer())
    }
}

// // //

pub fn prepare_data_for_send<T: Serialize>(data: &T) -> Result<MessageInfo, Error> {
    let used = sel4::with_ipc_buffer_mut(|ipc_buffer| {
        postcard::to_slice(data, ipc_buffer.msg_bytes_mut()).map(|used| used.len())
    })?;
    Ok(MessageInfoBuilder::default()
        .length(used.next_multiple_of(BYTES_PER_WORD) / BYTES_PER_WORD)
        .build())
}

pub fn prepare_bytes_for_send(bytes: &[u8]) -> Result<MessageInfo, Error> {
    sel4::with_ipc_buffer_mut(|ipc_buffer| {
        ipc_buffer.msg_bytes_mut()[..bytes.len()].copy_from_slice(bytes);
    });
    Ok(MessageInfoBuilder::default()
        .length(bytes.len().next_multiple_of(BYTES_PER_WORD) / BYTES_PER_WORD)
        .build())
}

fn recv_data<T: for<'a> Deserialize<'a>>(info: &MessageInfo) -> Result<T, Error> {
    sel4::with_ipc_buffer(|ipc_buffer| recv_data_with_ipc_buffer(info, ipc_buffer))
}

fn recv_data_with_ipc_buffer<T: for<'a> Deserialize<'a>>(
    info: &MessageInfo,
    ipc_buffer: &IpcBuffer,
) -> Result<T, Error> {
    postcard::from_bytes(recv_bytes_with_ipc_buffer(ipc_buffer, info)).map_err(Into::into)
}

fn recv_bytes_with_ipc_buffer<'a>(ipc_buffer: &'a IpcBuffer, info: &MessageInfo) -> &'a [u8] {
    &ipc_buffer.msg_bytes()[..BYTES_PER_WORD * usize::try_from(info.length()).unwrap()]
}

// // //

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    SeL4Error(sel4::Error),
    PostcardError(postcard::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SeL4Error(err) => write!(f, "seL4 error: {}", err),
            Self::PostcardError(err) => write!(f, "postcard error: {}", err),
        }
    }
}

impl From<sel4::Error> for Error {
    fn from(err: sel4::Error) -> Self {
        Self::SeL4Error(err)
    }
}

impl From<postcard::Error> for Error {
    fn from(err: postcard::Error) -> Self {
        Self::PostcardError(err)
    }
}
