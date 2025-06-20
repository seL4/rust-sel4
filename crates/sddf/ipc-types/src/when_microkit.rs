//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use sel4_microkit_base::{with_msg_regs, with_msg_regs_mut, Channel, MessageInfo};

use crate::{CallError, CallTarget, MessageReader, MessageWriter};

impl CallTarget for Channel {
    fn call<T, W: MessageWriter, R: MessageReader<T>>(
        &self,
        writer: W,
        reader: R,
    ) -> Result<T, CallError<W::Error, R::Error>> {
        let (label, n) =
            with_msg_regs_mut(|buf| writer.write_message(buf)).map_err(CallError::WriteError)?;
        let resp_info = self.pp_call(MessageInfo::new(label, n));
        with_msg_regs(|buf| reader.read_message(resp_info.label(), &buf[..resp_info.count()]))
            .map_err(CallError::ReadError)
    }
}
