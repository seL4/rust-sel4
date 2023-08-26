use sel4cp::MessageInfo;
use sel4cp_message::{MessageInfoExt as _, NoMessageValue, StatusMessageLabel};

use sel4cp_http_server_example_sp804_driver_interface_types::*;

pub struct TimerClient {
    channel: sel4cp::Channel,
}

impl TimerClient {
    pub fn new(channel: sel4cp::Channel) -> Self {
        Self { channel }
    }

    pub fn now(&self) -> Microseconds {
        let msg_info = self
            .channel
            .pp_call(MessageInfo::send(RequestTag::Now, NoMessageValue));
        assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));
        let NowResponse { micros } = msg_info.recv().unwrap();
        micros
    }

    pub fn set_timeout(&self, relative_micros: Microseconds) {
        let msg_info = self.channel.pp_call(MessageInfo::send(
            RequestTag::SetTimeout,
            SetTimeoutRequest { relative_micros },
        ));
        assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));
        let _ = msg_info.recv::<NoMessageValue>().unwrap();
    }

    #[allow(dead_code)]
    pub fn clear_timeout(&self) {
        let msg_info = self
            .channel
            .pp_call(MessageInfo::send(RequestTag::ClearTimeout, NoMessageValue));
        assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));
        let _ = msg_info.recv::<NoMessageValue>().unwrap();
    }
}
