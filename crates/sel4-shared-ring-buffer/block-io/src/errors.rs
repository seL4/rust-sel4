//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_shared_ring_buffer::PeerMisbehaviorError as SharedRingBuffersPeerMisbehaviorError;
use sel4_shared_ring_buffer_bookkeeping::slot_tracker::SlotTrackerError;

#[derive(Debug, Clone)]
pub enum ErrorOrUserError {
    Error(Error),
    UserError(UserError),
}

#[derive(Debug, Clone)]
pub enum UserError {
    InvalidRequestIndex,
    RequestStateMismatch,
    TooManyOutstandingRequests,
}

#[derive(Debug, Clone)]
pub enum Error {
    IOError(IOError),
    BounceBufferAllocationError,
    PeerMisbehaviorError(PeerMisbehaviorError),
}

#[derive(Debug, Clone)]
pub struct IOError;

#[derive(Debug, Clone)]
pub enum PeerMisbehaviorError {
    InvalidDescriptor,
    DescriptorMismatch,
    OutOfBoundsCookie,
    StateMismatch,
    SharedRingBuffersPeerMisbehaviorError(SharedRingBuffersPeerMisbehaviorError),
}

impl ErrorOrUserError {
    pub fn unwrap_error(self) -> Error {
        match self {
            Self::Error(err) => err,
            Self::UserError(err) => panic!(
                "called `ErrorOrUserError::unwrap_error()` on an `UserError` value: {:?}",
                err
            ),
        }
    }
}

impl From<Error> for ErrorOrUserError {
    fn from(err: Error) -> Self {
        Self::Error(err)
    }
}

impl From<UserError> for ErrorOrUserError {
    fn from(err: UserError) -> Self {
        Self::UserError(err)
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::IOError(err)
    }
}

impl From<IOError> for ErrorOrUserError {
    fn from(err: IOError) -> Self {
        Error::from(err).into()
    }
}

impl From<PeerMisbehaviorError> for Error {
    fn from(err: PeerMisbehaviorError) -> Self {
        Self::PeerMisbehaviorError(err)
    }
}

impl From<PeerMisbehaviorError> for ErrorOrUserError {
    fn from(err: PeerMisbehaviorError) -> Self {
        Error::from(err).into()
    }
}

impl From<SharedRingBuffersPeerMisbehaviorError> for PeerMisbehaviorError {
    fn from(err: SharedRingBuffersPeerMisbehaviorError) -> Self {
        Self::SharedRingBuffersPeerMisbehaviorError(err)
    }
}

impl From<SharedRingBuffersPeerMisbehaviorError> for Error {
    fn from(err: SharedRingBuffersPeerMisbehaviorError) -> Self {
        PeerMisbehaviorError::from(err).into()
    }
}

impl From<SharedRingBuffersPeerMisbehaviorError> for ErrorOrUserError {
    fn from(err: SharedRingBuffersPeerMisbehaviorError) -> Self {
        Error::from(err).into()
    }
}

impl From<SlotTrackerError> for UserError {
    fn from(err: SlotTrackerError) -> Self {
        match err {
            SlotTrackerError::OutOfBounds => UserError::InvalidRequestIndex,
            SlotTrackerError::StateMismatch => UserError::RequestStateMismatch,
        }
    }
}

impl From<SlotTrackerError> for ErrorOrUserError {
    fn from(err: SlotTrackerError) -> Self {
        UserError::from(err).into()
    }
}
