//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use alloc::vec;
use core::time::Duration;

use futures::future::{self, LocalBoxFuture};
use futures::task::LocalSpawnExt;
use rustls::pki_types::{PrivateKeyDer, UnixTime};
use rustls::version::TLS12;
use rustls::ServerConfig;

use sel4_async_block_io::{access::ReadOnly, constant_block_sizes, BlockIO};
use sel4_async_block_io_fat as fat;
use sel4_async_io::ReadExactError;
use sel4_async_network::{ManagedInterface, TcpSocket, TcpSocketError};
use sel4_async_network_rustls::{Error as AsyncRustlsError, ServerConnector};
use sel4_async_network_rustls_utils::TimeProviderImpl;
use sel4_async_single_threaded_executor::LocalSpawner;
use sel4_async_time::{Instant, TimerManager};

mod mime;
mod server;

use server::Server;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

#[allow(clippy::too_many_arguments)] // TODO
pub async fn run_server<
    T: BlockIO<ReadOnly, BlockSize = constant_block_sizes::BlockSize512> + Clone + 'static,
>(
    now_unix_time: Duration,
    now_fn: impl 'static + Send + Sync + Fn() -> Instant,
    _timers_ctx: TimerManager,
    network_ctx: ManagedInterface,
    fs_block_io: T,
    spawner: LocalSpawner,
    cert_pem: &str,
    priv_pem: &str,
    max_num_simultaneous_connections: usize,
) -> ! {
    let use_socket_for_http_closure: SocketUser<T> = Box::new({
        move |server, socket| {
            Box::pin(async move {
                use_socket_for_http(server, socket)
                    .await
                    .unwrap_or_else(|err| {
                        log::warn!("error: {err:?}");
                    })
            })
        }
    });

    let tls_config = Arc::new(mk_tls_config(cert_pem, priv_pem, now_unix_time, now_fn));

    let use_socket_for_https_closure: SocketUser<T> = Box::new({
        move |server, socket| {
            let tls_config = tls_config.clone();
            Box::pin(async move {
                use_socket_for_https(server, tls_config, socket)
                    .await
                    .unwrap_or_else(|err| {
                        log::warn!("error: {err:?}");
                    })
            })
        }
    });

    for f in [use_socket_for_http_closure, use_socket_for_https_closure].map(Rc::new) {
        for _ in 0..max_num_simultaneous_connections {
            spawner
                .spawn_local({
                    let network_ctx = network_ctx.clone();
                    let f = f.clone();
                    let fs_block_io = fs_block_io.clone();
                    async move {
                        loop {
                            let fs_block_io = fat::BlockIOWrapper::new(fs_block_io.clone());
                            let mut volume_manager =
                                fat::Volume::new(fs_block_io, fat::DummyTimeSource::new())
                                    .await
                                    .unwrap();
                            let dir = volume_manager.open_root_dir().unwrap();
                            let server = Server::new(volume_manager, dir);
                            let socket = network_ctx.new_tcp_socket_with_buffer_sizes(8192, 65535);
                            f(server, socket).await;
                        }
                    }
                })
                .unwrap()
        }
    }

    future::pending().await
}

type SocketUser<T> = Box<
    dyn Fn(
        Server<fat::BlockIOWrapper<T, ReadOnly>, fat::DummyTimeSource>,
        TcpSocket,
    ) -> LocalBoxFuture<'static, ()>,
>;

async fn use_socket_for_http<D: fat::BlockDevice + 'static, T: fat::TimeSource + 'static>(
    server: Server<D, T>,
    mut socket: TcpSocket,
) -> Result<(), ReadExactError<TcpSocketError>> {
    socket.accept(HTTP_PORT).await?;
    server.handle_connection(&mut socket).await?;
    socket.close();
    Ok(())
}

async fn use_socket_for_https<D: fat::BlockDevice + 'static, T: fat::TimeSource + 'static>(
    server: Server<D, T>,
    tls_config: Arc<ServerConfig>,
    mut socket: TcpSocket,
) -> Result<(), ReadExactError<AsyncRustlsError<TcpSocketError>>> {
    socket
        .accept(HTTPS_PORT)
        .await
        .map_err(AsyncRustlsError::TransitError)?;

    let mut conn = ServerConnector::from(tls_config).connect(socket)?.await?;

    server.handle_connection(&mut conn).await?;

    conn.into_io().close();

    Ok(())
}

fn mk_tls_config(
    cert_pem: &str,
    priv_pem: &str,
    now_unix_time: Duration,
    now_fn: impl 'static + Send + Sync + Fn() -> Instant,
) -> ServerConfig {
    let cert_der = match rustls_pemfile::read_one_from_slice(cert_pem.as_bytes())
        .unwrap()
        .unwrap()
        .0
    {
        rustls_pemfile::Item::X509Certificate(cert) => cert,
        _ => panic!(),
    };

    let key_der = match rustls_pemfile::read_one_from_slice(priv_pem.as_bytes())
        .unwrap()
        .unwrap()
        .0
    {
        rustls_pemfile::Item::Pkcs1Key(der) => PrivateKeyDer::Pkcs1(der),
        rustls_pemfile::Item::Pkcs8Key(der) => PrivateKeyDer::Pkcs8(der),
        rustls_pemfile::Item::Sec1Key(der) => PrivateKeyDer::Sec1(der),
        _ => panic!(),
    };

    let time_provider = TimeProviderImpl::new(UnixTime::since_unix_epoch(now_unix_time), now_fn);

    ServerConfig::builder_with_details(
        Arc::new(rustls::crypto::ring::default_provider()),
        Arc::new(time_provider),
    )
    .with_protocol_versions(&[&TLS12])
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(vec![cert_der], key_der)
    .unwrap()
}
