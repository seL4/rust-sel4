use alloc::sync::Arc;
use alloc::vec;
use core::str;

use smoltcp::time::Duration;
use smoltcp::wire::DnsQueryType;

use mbedtls::rng::CtrDrbg;
use mbedtls::ssl::async_io::AsyncIoExt;
use mbedtls::ssl::config::{Endpoint, Preset, Transport};
use mbedtls::ssl::{Config, Context};

use sel4_async_network::SharedNetwork;
use sel4_async_network_mbedtls::{insecure_dummy_rng, DbgCallbackBuilder, TcpSocketWrapper};
use sel4_async_timers::SharedTimers;

const CA_LIST: &[u8] = concat!(include_str!("../support/cacert.pem"), "\0").as_bytes();

pub async fn run(network_ctx: SharedNetwork, timers_ctx: SharedTimers) {
    timers_ctx.sleep(Duration::from_secs(1)).await;

    let query = network_ctx
        .dns_query("example.com", DnsQueryType::A)
        .await
        .unwrap();

    let mut socket = network_ctx.new_tcp_socket();
    socket.connect((query[0], 443), 44445).await.unwrap();

    let entropy = Arc::new(insecure_dummy_rng());
    let rng = Arc::new(CtrDrbg::new(entropy, None).unwrap());
    let mut config = Config::new(Endpoint::Client, Transport::Stream, Preset::Default);
    config.set_rng(rng);
    config.set_dbg_callback(
        DbgCallbackBuilder::default()
            .forward_log_level(log::Level::Warn)
            .build(),
    );
    config.set_ca_list(
        Arc::new(mbedtls::x509::Certificate::from_pem_multiple(CA_LIST).unwrap()),
        None,
    );

    let mut ctx = Context::new(Arc::new(config));

    ctx.establish_async(TcpSocketWrapper::new(socket), None)
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

    ctx.close_async().await.unwrap();
    ctx.take_io().unwrap().inner_mut().close().await.unwrap();
    drop(ctx);

    log::info!("client test complete");
}
