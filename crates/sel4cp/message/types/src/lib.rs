#![no_std]
#![feature(never_type)]

use core::fmt;
use core::mem;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use zerocopy::{AsBytes, FromBytes, Unalign};

#[cfg(feature = "postcard")]
mod when_postcard;

#[cfg(feature = "postcard")]
pub use when_postcard::MessageValueUsingPostcard;

#[cfg(target_pointer_width = "32")]
pub type MessageLabel = u32;

#[cfg(target_pointer_width = "64")]
pub type MessageLabel = u64;

pub trait MessageValueSend {
    type Error;

    fn write_message_value(self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

pub trait MessageValueRecv: Sized {
    type Error;

    fn read_message_value(buf: &[u8]) -> Result<Self, Self::Error>;
}

pub trait MessageSend {
    type Label: Into<MessageLabel>;

    type Error;

    fn write_message(self, buf: &mut [u8]) -> Result<(Self::Label, usize), Self::Error>;
}

pub trait MessageRecv: Sized {
    type Label: TryFrom<MessageLabel>;

    type Error;

    fn read_message(label: Self::Label, buf: &[u8]) -> Result<Self, Self::Error>;
}

// // //

impl<T: AsBytes> MessageValueSend for T {
    type Error = SendAsBytesError;

    fn write_message_value(self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.write_to_prefix(buf)
            .ok_or(SendAsBytesError::ValueTooLarge)?;
        Ok(mem::size_of_val(&self))
    }
}

impl<T: FromBytes + Copy> MessageValueRecv for T {
    type Error = RecvFromBytesError;

    fn read_message_value(buf: &[u8]) -> Result<Self, Self::Error> {
        Unalign::<T>::read_from_prefix(buf)
            .ok_or(RecvFromBytesError::MessageTooShort)
            .map(|unalign| unalign.get())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SendAsBytesError {
    ValueTooLarge,
}

#[derive(Copy, Clone, Debug)]
pub enum RecvFromBytesError {
    MessageTooShort,
}

// // //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EmptyMessageValue;

impl MessageValueSend for EmptyMessageValue {
    type Error = !;

    fn write_message_value(self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

impl MessageValueRecv for EmptyMessageValue {
    type Error = !;

    fn read_message_value(_buf: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}

// // //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EmptyMessage;

impl MessageSend for EmptyMessage {
    type Label = DefaultMessageLabel;

    type Error = !;

    fn write_message(self, buf: &mut [u8]) -> Result<(Self::Label, usize), Self::Error> {
        TriviallyLabeled(EmptyMessageValue).write_message(buf)
    }
}

impl MessageRecv for EmptyMessage {
    type Label = DefaultMessageLabel;

    type Error = !;

    fn read_message(label: Self::Label, buf: &[u8]) -> Result<Self, Self::Error> {
        TriviallyLabeled::<EmptyMessageValue>::read_message(label, buf).map(|_| Self)
    }
}

// // //

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct DefaultMessageLabel;

impl From<DefaultMessageLabel> for MessageLabel {
    fn from(_: DefaultMessageLabel) -> Self {
        0
    }
}

impl TryFrom<MessageLabel> for DefaultMessageLabel {
    type Error = TryFromDefaultMessageLabelError;

    fn try_from(val: MessageLabel) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self),
            _ => Err(TryFromDefaultMessageLabelError(())),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TryFromDefaultMessageLabelError(());

impl fmt::Display for TryFromDefaultMessageLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected label value of 0")
    }
}

// // //

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TriviallyLabeled<T>(pub T);

impl<T: MessageValueSend> MessageSend for TriviallyLabeled<T> {
    type Label = DefaultMessageLabel;

    type Error = <T as MessageValueSend>::Error;

    fn write_message(self, buf: &mut [u8]) -> Result<(Self::Label, usize), Self::Error> {
        Ok((DefaultMessageLabel, self.0.write_message_value(buf)?))
    }
}

impl<T: MessageValueRecv> MessageRecv for TriviallyLabeled<T> {
    type Label = DefaultMessageLabel;

    type Error = <T as MessageValueRecv>::Error;

    fn read_message(_: Self::Label, buf: &[u8]) -> Result<Self, Self::Error> {
        T::read_message_value(buf).map(TriviallyLabeled)
    }
}

// // //

// TODO unecessary?

impl<T: MessageValueSend, E: MessageValueSend> MessageSend for Result<T, E> {
    type Label = ResultMessageLabel;

    type Error = ResultMessageValueError<T::Error, E::Error>;

    fn write_message(self, buf: &mut [u8]) -> Result<(Self::Label, usize), Self::Error> {
        Ok(match self {
            Ok(ok) => (
                Self::Label::Ok,
                ok.write_message_value(buf).map_err(Self::Error::Ok)?,
            ),
            Err(err) => (
                Self::Label::Err,
                err.write_message_value(buf).map_err(Self::Error::Err)?,
            ),
        })
    }
}

impl<T: MessageValueRecv, E: MessageValueRecv> MessageRecv for Result<T, E> {
    type Label = ResultMessageLabel;

    type Error = ResultMessageValueError<T::Error, E::Error>;

    fn read_message(label: Self::Label, buf: &[u8]) -> Result<Self, Self::Error> {
        match label {
            Self::Label::Ok => MessageValueRecv::read_message_value(buf)
                .map(Ok)
                .map_err(Self::Error::Ok),
            Self::Label::Err => MessageValueRecv::read_message_value(buf)
                .map(Err)
                .map_err(Self::Error::Err),
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(target_pointer_width = "32", repr(u32))]
#[cfg_attr(target_pointer_width = "64", repr(u64))]
pub enum ResultMessageLabel {
    Ok = 0,
    Err = 1,
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum ResultMessageValueError<E1, E2> {
    Ok(E1),
    Err(E2),
}
