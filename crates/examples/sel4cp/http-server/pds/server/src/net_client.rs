use sel4cp::MessageInfo;
use sel4cp_message::MessageInfoExt as _;

use sel4cp_http_server_example_virtio_net_driver_interface_types::*;

pub struct NetClient {
    channel: sel4cp::Channel,
}

impl NetClient {
    pub fn new(channel: sel4cp::Channel) -> Self {
        Self { channel }
    }

    pub fn get_mac_address(&self) -> MacAddress {
        let req = Request::GetMacAddress;
        let resp: GetMacAddressResponse = self
            .channel
            .pp_call(MessageInfo::send_using_postcard(req).unwrap())
            .recv_using_postcard()
            .unwrap();
        resp.mac_address
    }
}
