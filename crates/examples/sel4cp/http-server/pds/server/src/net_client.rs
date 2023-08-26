use sel4cp::MessageInfo;
use sel4cp_message::{MessageInfoExt as _, NoMessageValue, StatusMessageLabel};

use sel4cp_http_server_example_virtio_net_driver_interface_types::*;

pub struct NetClient {
    channel: sel4cp::Channel,
}

impl NetClient {
    pub fn new(channel: sel4cp::Channel) -> Self {
        Self { channel }
    }

    pub fn get_mac_address(&self) -> MacAddress {
        let msg_info = self
            .channel
            .pp_call(MessageInfo::send(RequestTag::GetMacAddress, NoMessageValue));
        assert_eq!(msg_info.label().try_into(), Ok(StatusMessageLabel::Ok));
        let GetMacAddressResponse { mac_address } = msg_info.recv().unwrap();
        mac_address
    }
}
