//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::ops::Range;
use core::slice::{Chunks, ChunksMut};

use crate::access::{Access, ReadAccess, ReadOnly, WriteAccess, WriteOnly};

pub enum Operation<'a, A: Access> {
    Read {
        buf: &'a mut [u8],
        witness: A::ReadWitness,
    },
    Write {
        buf: &'a [u8],
        witness: A::WriteWitness,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperationType {
    Read,
    Write,
}

impl OperationType {
    pub fn is_read(self) -> bool {
        self == Self::Read
    }

    pub fn is_write(self) -> bool {
        self == Self::Write
    }
}

impl<'a, A: Access> Operation<'a, A> {
    pub fn with_read_access<A1: ReadAccess + Access<WriteWitness = A::WriteWitness>>(
        &'a mut self,
    ) -> Operation<'a, A1> {
        match self {
            Self::Read { buf, .. } => Operation::Read {
                buf,
                witness: A1::READ_WITNESS,
            },
            Self::Write { buf, witness } => Operation::Write {
                buf,
                witness: *witness,
            },
        }
    }

    pub fn with_write_access<A1: WriteAccess + Access<ReadWitness = A::ReadWitness>>(
        &'a mut self,
    ) -> Operation<'a, A1> {
        match self {
            Self::Read { buf, witness } => Operation::Read {
                buf,
                witness: *witness,
            },
            Self::Write { buf, .. } => Operation::Write {
                buf,
                witness: A1::WRITE_WITNESS,
            },
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Read { buf, .. } => buf.len(),
            Self::Write { buf, .. } => buf.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn ty(&self) -> OperationType {
        match self {
            Self::Read { .. } => OperationType::Read,
            Self::Write { .. } => OperationType::Write,
        }
    }

    pub fn index(&'a mut self, index: Range<usize>) -> Self {
        match self {
            Self::Read { buf, witness } => Self::Read {
                buf: &mut buf[index],
                witness: *witness,
            },
            Self::Write { buf, witness } => Self::Write {
                buf: &buf[index],
                witness: *witness,
            },
        }
    }

    pub fn split_at(&'a mut self, mid: usize) -> (Self, Self) {
        match self {
            Self::Read { buf, witness } => {
                let (left, right) = buf.split_at_mut(mid);
                let left = Self::Read {
                    buf: left,
                    witness: *witness,
                };
                let right = Self::Read {
                    buf: right,
                    witness: *witness,
                };
                (left, right)
            }
            Self::Write { buf, witness } => {
                let (left, right) = buf.split_at(mid);
                let left = Self::Write {
                    buf: left,
                    witness: *witness,
                };
                let right = Self::Write {
                    buf: right,
                    witness: *witness,
                };
                (left, right)
            }
        }
    }

    pub fn chunks(&'a mut self, chunk_size: usize) -> impl Iterator<Item = Operation<'a, A>> {
        match self {
            Self::Read { buf, witness } => OperationChunks::Read {
                it: buf.chunks_mut(chunk_size),
                witness: *witness,
            },
            Self::Write { buf, witness } => OperationChunks::Write {
                it: buf.chunks(chunk_size),
                witness: *witness,
            },
        }
    }
}

enum OperationChunks<'a, A: Access> {
    Read {
        it: ChunksMut<'a, u8>,
        witness: A::ReadWitness,
    },
    Write {
        it: Chunks<'a, u8>,
        witness: A::WriteWitness,
    },
}

impl<'a, A: Access> Iterator for OperationChunks<'a, A> {
    type Item = Operation<'a, A>;

    fn next(&mut self) -> Option<Operation<'a, A>> {
        match self {
            Self::Read { it, witness } => it.next().map(|buf| Operation::Read {
                buf,
                witness: *witness,
            }),
            Self::Write { it, witness } => it.next().map(|buf| Operation::Write {
                buf,
                witness: *witness,
            }),
        }
    }
}

impl<'a> Operation<'a, ReadOnly> {
    pub fn as_read(&'a mut self) -> &'a mut [u8] {
        match self {
            Self::Read { buf, .. } => buf,
            Self::Write { witness, .. } => *witness,
        }
    }
}

impl<'a> Operation<'a, WriteOnly> {
    pub fn as_write(&'a self) -> &'a [u8] {
        match self {
            Self::Read { witness, .. } => *witness,
            Self::Write { buf, .. } => buf,
        }
    }
}

impl<'a, A: ReadAccess> Operation<'a, A> {
    pub fn read(buf: &'a mut [u8]) -> Self {
        Self::Read {
            buf,
            witness: A::READ_WITNESS,
        }
    }
}

impl<'a, A: WriteAccess> Operation<'a, A> {
    pub fn write(buf: &'a [u8]) -> Self {
        Self::Write {
            buf,
            witness: A::WRITE_WITNESS,
        }
    }
}
