use crate::{newtype_methods, sys, Word};

/// Corresponds to `seL4_MessageInfo_t`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageInfo(sys::seL4_MessageInfo);

impl MessageInfo {
    newtype_methods!(sys::seL4_MessageInfo);

    pub fn new(label: Word, caps_unwrapped: usize, extra_caps: usize, length: usize) -> Self {
        Self::from_inner(sys::seL4_MessageInfo::new(
            label,
            caps_unwrapped.try_into().unwrap(),
            extra_caps.try_into().unwrap(),
            length.try_into().unwrap(),
        ))
    }

    pub fn label(&self) -> Word {
        self.inner().get_label()
    }

    pub const fn label_width() -> usize {
        sys::seL4_MessageInfo::width_of_label()
    }

    pub fn caps_unwrapped(&self) -> usize {
        self.inner().get_capsUnwrapped().try_into().unwrap()
    }

    pub fn extra_caps(&self) -> usize {
        self.inner().get_extraCaps().try_into().unwrap()
    }

    pub fn length(&self) -> usize {
        self.inner().get_length().try_into().unwrap()
    }
}

impl From<MessageInfoBuilder> for MessageInfo {
    fn from(builder: MessageInfoBuilder) -> Self {
        builder.build()
    }
}

/// Helper for constructing [`MessageInfo`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageInfoBuilder {
    label: Word,
    caps_unwrapped: usize,
    extra_caps: usize,
    length: usize,
}

impl MessageInfoBuilder {
    pub fn build(self) -> MessageInfo {
        MessageInfo::new(
            self.label,
            self.caps_unwrapped,
            self.extra_caps,
            self.length,
        )
    }

    pub fn label(mut self, label: Word) -> Self {
        self.label = label;
        self
    }

    pub fn caps_unwrapped(mut self, caps_unwrapped: usize) -> Self {
        self.caps_unwrapped = caps_unwrapped;
        self
    }

    pub fn extra_caps(mut self, extra_caps: usize) -> Self {
        self.extra_caps = extra_caps;
        self
    }

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }
}
