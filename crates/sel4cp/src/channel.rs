#![allow(dead_code)]

pub(crate) type Slot = usize;

const BASE_OUTPUT_NOTIFICATION_CAP: Slot = 10;
const BASE_ENDPOINT_CAP: Slot = 74;
const BASE_IRQ_CAP: Slot = 138;

const MAX_CHANNELS: Slot = 63;

pub(crate) const fn slot_to_local_cptr<T: sel4::CapType>(slot: Slot) -> sel4::LocalCPtr<T> {
    sel4::LocalCPtr::from_bits(slot as sel4::CPtrBits)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Channel {
    index: usize,
}

impl Channel {
    pub const fn new(index: usize) -> Self {
        Self { index }
    }

    fn local_cptr<T: sel4::CapType>(&self, offset: Slot) -> sel4::LocalCPtr<T> {
        slot_to_local_cptr(offset + self.index)
    }

    pub fn notify(&self) {
        self.local_cptr::<sel4::cap_type::Notification>(BASE_OUTPUT_NOTIFICATION_CAP)
            .signal()
    }

    // TODO don't expose sel4::Error
    pub fn irq_ack(&self) -> Result<(), sel4::Error> {
        self.local_cptr::<sel4::cap_type::IRQHandler>(BASE_IRQ_CAP)
            .irq_handler_ack()
    }

    pub fn pp_call(&self, msg_info: MessageInfo) -> MessageInfo {
        MessageInfo::from_sel4(
            self.local_cptr::<sel4::cap_type::Endpoint>(BASE_ENDPOINT_CAP)
                .call(msg_info.into_sel4()),
        )
    }
}

#[derive(Debug)]
pub struct MessageInfo {
    inner: sel4::MessageInfo,
}

pub type Label = sel4::Word;

impl MessageInfo {
    pub(crate) fn from_sel4(inner: sel4::MessageInfo) -> Self {
        Self { inner }
    }

    pub(crate) fn into_sel4(self) -> sel4::MessageInfo {
        self.inner
    }

    pub fn new(label: Label, count: usize) -> Self {
        Self::from_sel4(sel4::MessageInfo::new(label, 0, 0, count))
    }

    pub fn label(&self) -> Label {
        self.inner.label()
    }

    pub fn count(&self) -> usize {
        self.inner.length()
    }
}

pub type MessageValue = sel4::Word;

pub fn with_msg_regs<T>(f: impl FnOnce(&[MessageValue]) -> T) -> T {
    sel4::with_borrow_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_regs()))
}

pub fn with_msg_regs_mut<T>(f: impl FnOnce(&mut [MessageValue]) -> T) -> T {
    sel4::with_borrow_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_regs_mut()))
}

pub fn with_msg_bytes<T>(f: impl FnOnce(&[u8]) -> T) -> T {
    sel4::with_borrow_ipc_buffer(|ipc_buffer| f(ipc_buffer.msg_bytes()))
}

pub fn with_msg_bytes_mut<T>(f: impl FnOnce(&mut [u8]) -> T) -> T {
    sel4::with_borrow_ipc_buffer_mut(|ipc_buffer| f(ipc_buffer.msg_bytes_mut()))
}

pub fn set_mr(i: usize, value: MessageValue) {
    with_msg_regs_mut(|regs| regs[i] = value)
}

pub fn get_mr(i: usize) -> MessageValue {
    with_msg_regs(|regs| regs[i])
}

// pub const DOES_HAVE_NOTIFICATION_IN: bool = true;
// pub const DOES_HAVE_NOTIFICATION_OUT: bool = true;
// pub const DOES_HAVE_PP_IN: bool = true;
// pub const DOES_HAVE_PP_OUT: bool = true;
// pub const DOES_HAVE_IRQ: bool = true;

// pub struct Channel<
//     const HAS_NOTIFICATION_IN: bool = false,
//     const HAS_NOTIFICATION_OUT: bool = false,
//     const HAS_PP_IN: bool = false,
//     const HAS_PP_OUT: bool = false,
//     const HAS_IRQ: bool = false,
// >(usize);

// impl<
//         const HAS_NOTIFICATION_IN: bool,
//         const HAS_PP_IN: bool,
//         const HAS_PP_OUT: bool,
//         const HAS_IRQ: bool,
//     > Channel<HAS_NOTIFICATION_IN, DOES_HAVE_NOTIFICATION_OUT, HAS_PP_IN, HAS_PP_OUT, HAS_IRQ>
// {
//     pub fn notify(&self) {
//     }
// }
