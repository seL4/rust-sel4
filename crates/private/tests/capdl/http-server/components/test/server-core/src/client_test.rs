use alloc::borrow::Cow;
use alloc::sync::Arc;
use alloc::vec;
use core::str;

use smoltcp::time::Duration;
use smoltcp::wire::DnsQueryType;

use mbedtls::rng::CtrDrbg;
use mbedtls::ssl::config::{Endpoint, Preset, Transport};
use mbedtls::ssl::{Config, Context};

use sel4_async_network::SharedNetwork;
use sel4_async_network_mbedtls::{
    insecure_dummy_entropy, mbedtls, seed_insecure_dummy_entropy, ContextWrapper, TcpSocketWrapper,
};
use sel4_async_timers::SharedTimers;
use sel4_panicking_env::debug_print;

const CA_LIST: &[u8] = concat!(include_str!("../support/cacert.pem"), "\0").as_bytes();

pub async fn run(network_ctx: SharedNetwork, timers_ctx: SharedTimers) {
    {
        use sel4_newlib::*;

        set_static_heap_for_sbrk({
            static HEAP: StaticHeap<{ 1024 * 1024 }> = StaticHeap::new();
            &HEAP
        });

        let mut impls = Implementations::default();
        impls._sbrk = Some(sbrk_with_static_heap);
        impls._write = Some(write_with_debug_put_char);
        set_implementations(impls)
    }

    unsafe {
        mbedtls::set_global_debug_threshold(3);
    }

    seed_insecure_dummy_entropy(0);

    timers_ctx.sleep(Duration::from_secs(1)).await;

    let query = network_ctx
        .dns_query("example.com", DnsQueryType::A)
        .await
        .unwrap();

    let mut socket = network_ctx.new_tcp_socket();
    socket.connect((query[0], 443), 44445).await.unwrap();

    let config = {
        let mut this = Config::new(Endpoint::Client, Transport::Stream, Preset::Default);
        let entropy = Arc::new(insecure_dummy_entropy());
        let rng = Arc::new(CtrDrbg::new(entropy, None).unwrap());
        this.set_rng(rng);
        this.set_dbg_callback(dbg_callback_brief);
        this.set_ca_list(
            Arc::new(mbedtls::x509::Certificate::from_pem_multiple(CA_LIST).unwrap()),
            None,
        );
        this
    };

    let mut ctx = ContextWrapper::new(Context::new(Arc::new(config)));

    ctx.establish(TcpSocketWrapper::new(socket), None)
        .await
        .unwrap();

    ctx.send_all(b"GET / HTTP/1.1\r\n\r\n").await.unwrap();

    let mut buf = vec![0; 4096];
    loop {
        let n = ctx.recv(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        log::info!("{}", str::from_utf8(&buf[..n]).unwrap());
    }

    ctx.close().await.unwrap();

    drop(ctx);

    log::info!("client test complete");
}

fn dbg_callback_brief(_level: i32, _file: Cow<'_, str>, _line: i32, message: Cow<'_, str>) {
    debug_print!("{}", message);
}

#[allow(dead_code)]
fn dbg_callback_detailed(level: i32, file: Cow<'_, str>, line: i32, message: Cow<'_, str>) {
    let prefix = "[mbedtls]";
    log::info!("{}({}) {}:{} {}", prefix, level, file, line, message);
}
