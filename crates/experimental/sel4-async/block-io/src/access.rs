//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub trait Access: AccessSealed {
    type ReadWitness: Witness;
    type WriteWitness: Witness;
}

#[derive(Copy, Clone)]
pub enum Absurdity {}

impl Absurdity {
    pub(crate) fn absurd<T>(self) -> T {
        match self {}
    }
}

pub trait Witness: Sized + Copy + Unpin {
    const TRY_WITNESS: Option<Self>;
}

impl Witness for () {
    const TRY_WITNESS: Option<Self> = Some(());
}

impl Witness for Absurdity {
    const TRY_WITNESS: Option<Self> = None;
}

pub trait ReadAccess: Access {
    const READ_WITNESS: Self::ReadWitness;
}

pub trait WriteAccess: Access {
    const WRITE_WITNESS: Self::WriteWitness;
}

use sealing::AccessSealed;

mod sealing {
    use super::{ReadOnly, ReadWrite, WriteOnly};

    pub trait AccessSealed {}

    impl AccessSealed for ReadOnly {}
    impl AccessSealed for WriteOnly {}
    impl AccessSealed for ReadWrite {}
}

pub enum ReadOnly {}

impl Access for ReadOnly {
    type ReadWitness = ();
    type WriteWitness = Absurdity;
}

impl ReadAccess for ReadOnly {
    const READ_WITNESS: Self::ReadWitness = ();
}

pub enum WriteOnly {}

impl Access for WriteOnly {
    type ReadWitness = Absurdity;
    type WriteWitness = ();
}

impl WriteAccess for WriteOnly {
    const WRITE_WITNESS: Self::WriteWitness = ();
}

pub enum ReadWrite {}

impl Access for ReadWrite {
    type ReadWitness = ();
    type WriteWitness = ();
}

impl ReadAccess for ReadWrite {
    const READ_WITNESS: Self::ReadWitness = ();
}

impl WriteAccess for ReadWrite {
    const WRITE_WITNESS: Self::WriteWitness = ();
}
