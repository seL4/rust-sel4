//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::sync::Arc;
use alloc::vec;
use core::str;
use core::time::Duration;

use rustls::pki_types::{ServerName, UnixTime};
use rustls::time_provider::TimeProvider;
use rustls::version::TLS12;
use rustls::{ClientConfig, RootCertStore};
use smoltcp::wire::{DnsQueryType, IpAddress};

use sel4_async_network::ManagedInterface;
use sel4_async_network_rustls::async_io::{AsyncIOExt, ClientConnector, TcpSocketWrapper};
use sel4_async_network_rustls::{GetCurrentTimeImpl, NoServerCertVerifier};
use sel4_async_time::{Instant, TimerManager};

// TODO
const NOW: u64 = 1704284617;

const DOMAIN: &str = "example.com";
const PORT: u16 = 443;

// const DOMAIN: &str = "localhost";
// const PORT: u16 = 44330;

pub async fn run(
    now_fn: impl 'static + Send + Sync + Fn() -> Instant,
    network_ctx: ManagedInterface,
    timers_ctx: TimerManager,
) {
    timers_ctx
        .sleep_until(now_fn() + Duration::from_secs(1))
        .await;

    let addr = {
        if DOMAIN == "localhost" {
            IpAddress::v4(127, 0, 0, 1)
        } else {
            network_ctx
                .dns_query(DOMAIN, DnsQueryType::A)
                .await
                .unwrap()[0]
        }
    };

    let mut socket = network_ctx.new_tcp_socket();
    socket.connect((addr, PORT), 44445).await.unwrap();

    let config = {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let mut this = ClientConfig::builder_with_protocol_versions(&[&TLS12])
            .with_root_certificates(root_store)
            .with_no_client_auth();
        this.enable_early_data = false;
        this.time_provider = TimeProvider::new(GetCurrentTimeImpl::new(
            UnixTime::since_unix_epoch(Duration::from_secs(NOW)),
            now_fn,
        ));

        if DOMAIN == "localhost" {
            let mut dangerous_config = ClientConfig::dangerous(&mut this);
            dangerous_config.set_certificate_verifier(Arc::new(NoServerCertVerifier));
        }

        this
    };

    let mut conn = ClientConnector::from(Arc::new(config))
        .connect(
            ServerName::DnsName(DOMAIN.try_into().unwrap()),
            TcpSocketWrapper::new(socket),
        )
        .unwrap()
        .await
        .unwrap();

    conn.write_all(b"GET / HTTP/1.1\r\n").await.unwrap();
    conn.write_all(b"Host: example.com\r\n").await.unwrap();
    conn.write_all(b"\r\n").await.unwrap();
    conn.flush().await.unwrap();

    const BUF_SIZE: usize = 1024 * 64;
    // const BUF_SIZE: usize = 4096;

    let mut buf = vec![0; BUF_SIZE];
    loop {
        let n = conn.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        log::info!("{}", str::from_utf8(&buf[..n]).unwrap());
    }

    // ctx.close_async().await.unwrap();
    // ctx.take_io().unwrap().inner_mut().close().await.unwrap();
    // drop(ctx);

    log::info!("client test complete");
}
