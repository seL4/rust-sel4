use sel4cp::MessageInfo;
use sel4cp_message::{types::EmptyMessage, MessageInfoExt as _};

use sel4cp_http_server_example_sp804_driver_interface_types::*;

pub struct TimerClient {
    channel: sel4cp::Channel,
}

impl TimerClient {
    pub fn new(channel: sel4cp::Channel) -> Self {
        Self { channel }
    }

    pub fn now(&self) -> Microseconds {
        let req = Request::Now;
        let resp: NowResponse = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();
        resp.micros
    }

    pub fn set_timeout(&self, relative_micros: Microseconds) {
        let req = Request::SetTimeout { relative_micros };
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_empty()
            .unwrap();
    }

    #[allow(dead_code)]
    pub fn clear_timeout(&self) {
        let req = Request::ClearTimeout;
        self.channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_empty()
            .unwrap();
    }
}
