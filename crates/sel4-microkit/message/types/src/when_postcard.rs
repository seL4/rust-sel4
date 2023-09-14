use serde::{Deserialize, Serialize};

use crate::{MessageValueRecv, MessageValueSend};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MessageValueUsingPostcard<T>(pub T);

impl<T: Serialize> MessageValueSend for MessageValueUsingPostcard<T> {
    type Error = postcard::Error;

    fn write_message_value(self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        postcard::to_slice(&self.0, buf).map(|used| used.len())
    }
}

impl<T: for<'a> Deserialize<'a>> MessageValueRecv for MessageValueUsingPostcard<T> {
    type Error = postcard::Error;

    fn read_message_value(buf: &[u8]) -> Result<Self, Self::Error> {
        postcard::from_bytes(buf).map(MessageValueUsingPostcard)
    }
}
