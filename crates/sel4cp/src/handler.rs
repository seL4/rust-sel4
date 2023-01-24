use crate::{slot_to_local_cptr, Channel, MessageInfo};

const INPUT_CAP: sel4::Endpoint = slot_to_local_cptr(1);
const REPLY_CAP: sel4::Reply = slot_to_local_cptr(4);

pub trait Handler {
    type Error;

    fn notified(&mut self, channel: Channel) -> Result<(), Self::Error> {
        panic!("unexpected notification from {:?}", channel)
    }

    fn protected(
        &mut self,
        channel: Channel,
        _msg_info: MessageInfo,
    ) -> Result<MessageInfo, Self::Error> {
        panic!("unexpected protected procedure call from {:?}", channel)
    }

    fn run(&mut self) -> Result<!, Self::Error> {
        let mut reply_tag: Option<MessageInfo> = None;
        loop {
            let (tag, badge) = match reply_tag {
                Some(tag) => INPUT_CAP.reply_recv(tag.into_sel4(), REPLY_CAP),
                None => INPUT_CAP.recv(REPLY_CAP),
            };
            let tag = MessageInfo::from_sel4(tag);

            let is_endpoint = badge >> 63 != 0;

            reply_tag = if is_endpoint {
                Some(self.protected(Channel::new((badge & 0x3f).try_into().unwrap()), tag)?)
            } else {
                let mut badge_bits = badge;
                while badge_bits != 0 {
                    let i = badge_bits.leading_zeros();
                    self.notified(Channel::new(i.try_into().unwrap()))?;
                    badge_bits &= !(1 << i);
                }
                None
            };
        }
    }
}

pub struct NullHandler(());

impl NullHandler {
    pub fn new() -> Self {
        Self(())
    }
}

impl Handler for NullHandler {
    type Error = !;
}
