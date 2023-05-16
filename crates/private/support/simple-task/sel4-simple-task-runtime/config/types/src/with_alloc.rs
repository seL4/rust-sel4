use core::mem;
use core::ops::Range;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use zerocopy::AsBytes;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    Address, CPtrBits, Head, RuntimeConfig, RuntimeThreadConfig, Thread, ZerocopyOptionWord,
    ZerocopyOptionWordRange, ZerocopyWord, ZerocopyWordRange,
};

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RuntimeConfigForPacking<T> {
    pub static_heap: Option<Range<Address>>,
    pub static_heap_mutex_notification: Option<CPtrBits>,
    pub idle_notification: Option<CPtrBits>,
    pub threads: Vec<RuntimeThreadConfigForPacking>,
    pub image_identifier: Option<String>,
    pub arg: T,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RuntimeThreadConfigForPacking {
    ipc_buffer_addr: Address,
    endpoint: Option<CPtrBits>,
    reply_authority: Option<CPtrBits>,
}

impl<T> RuntimeConfigForPacking<T> {
    pub fn traverse<U, V>(
        self,
        f: impl FnOnce(T) -> Result<U, V>,
    ) -> Result<RuntimeConfigForPacking<U>, V> {
        Ok(RuntimeConfigForPacking {
            static_heap: self.static_heap,
            static_heap_mutex_notification: self.static_heap_mutex_notification,
            idle_notification: self.idle_notification,
            threads: self.threads,
            image_identifier: self.image_identifier,
            arg: f(self.arg)?,
        })
    }
}

impl<T: AsRef<[u8]>> RuntimeConfigForPacking<T> {
    pub fn pack(&self) -> Vec<u8> {
        let mut builder = BlobBuilder::new(mem::size_of::<Head>());
        let threads = self
            .threads
            .iter()
            .map(RuntimeThreadConfigForPacking::pack)
            .collect::<Vec<_>>();
        builder.align(mem::align_of::<RuntimeThreadConfig>());
        let threads =
            ZerocopyWordRange::try_from_native(&builder.append(threads.as_slice().as_bytes()))
                .unwrap();
        let image_identifier = ZerocopyOptionWordRange::try_from_native(
            &self
                .image_identifier
                .as_ref()
                .map(|image_identifier| builder.append(image_identifier.as_bytes())),
        )
        .unwrap();
        let arg = ZerocopyWordRange::try_from_native(&builder.append(self.arg.as_ref())).unwrap();
        let blob_buf = builder.build();
        let header = Head {
            static_heap: ZerocopyOptionWordRange::try_from_native(&self.static_heap).unwrap(),
            static_heap_mutex_notification: ZerocopyOptionWord::try_from_native(
                &self.static_heap_mutex_notification,
            )
            .unwrap(),
            idle_notification: ZerocopyOptionWord::try_from_native(&self.idle_notification)
                .unwrap(),
            threads,
            image_identifier,
            arg,
        };
        let mut buf = header.as_bytes().to_vec();
        buf.extend(&blob_buf);
        buf
    }
}

impl<'a> RuntimeConfigForPacking<&'a [u8]> {
    pub fn unpack(config: &'a RuntimeConfig) -> Self {
        RuntimeConfigForPacking {
            static_heap: config.static_heap(),
            static_heap_mutex_notification: config.static_heap_mutex_notification(),
            idle_notification: config.idle_notification(),
            threads: config
                .threads()
                .iter()
                .map(|thread| RuntimeThreadConfigForPacking::unpack(&thread))
                .collect::<Vec<_>>(),
            image_identifier: config.image_identifier().map(ToOwned::to_owned),
            arg: config.arg(),
        }
    }
}

impl RuntimeThreadConfigForPacking {
    pub fn pack(&self) -> RuntimeThreadConfig {
        RuntimeThreadConfig {
            inner: Thread {
                ipc_buffer_addr: self.ipc_buffer_addr.into(),
                endpoint: ZerocopyOptionWord::from(self.endpoint.map(ZerocopyWord::new).as_ref()),
                reply_authority: ZerocopyOptionWord::from(
                    self.reply_authority.map(ZerocopyWord::new).as_ref(),
                ),
            },
        }
    }

    pub fn unpack(config: &RuntimeThreadConfig) -> Self {
        RuntimeThreadConfigForPacking {
            ipc_buffer_addr: config.ipc_buffer_addr(),
            endpoint: config.endpoint(),
            reply_authority: config.reply_authority(),
        }
    }
}

struct BlobBuilder {
    start: usize,
    buf: Vec<u8>,
}

impl BlobBuilder {
    fn new(start: usize) -> Self {
        Self { start, buf: vec![] }
    }

    fn build(self) -> Vec<u8> {
        self.buf
    }

    fn cursor(&self) -> usize {
        self.start + self.buf.len()
    }

    fn align(&mut self, alignment: usize) {
        let padding = self.cursor().next_multiple_of(alignment) - self.cursor();
        let new_len = self.buf.len() + padding;
        self.buf.resize(new_len, 0);
    }

    fn append(&mut self, bytes: &[u8]) -> Range<usize> {
        let start = self.cursor();
        self.buf.extend(bytes);
        let end = self.cursor();
        start..end
    }
}
