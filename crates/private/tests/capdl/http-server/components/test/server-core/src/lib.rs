#![no_std]
#![feature(pattern)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;

use futures::future::{self, LocalBoxFuture};
use futures::task::LocalSpawnExt;

use mbedtls::ssl::async_io::ClosedError;

use sel4_async_network::{SharedNetwork, TcpSocketError};
use sel4_async_network_mbedtls::{
    insecure_dummy_rng, mbedtls, seed_insecure_dummy_rng, DbgCallbackBuilder, TcpSocketWrapper,
};
use sel4_async_single_threaded_executor::LocalSpawner;
use sel4_async_timers::SharedTimers;
use tests_capdl_http_server_components_test_cpiofs as cpiofs;

mod server;

use server::Server;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

const NUM_SIMULTANEOUS_CONNECTIONS: usize = 100;

type SocketUser = Box<dyn Fn(TcpSocketWrapper) -> LocalBoxFuture<'static, ()>>;

pub async fn run_server<T: cpiofs::IO + 'static>(
    network_ctx: SharedNetwork,
    _timers_ctx: SharedTimers,
    blk_device: T,
    spawner: LocalSpawner,
    cert_pem: &str,
    priv_pem: &str,
) -> ! {
    #[cfg(feature = "debug")]
    unsafe {
        mbedtls::set_global_debug_threshold(1);
    }

    seed_insecure_dummy_rng(0);

    let index = cpiofs::Index::create(blk_device).await;

    let server = Rc::new(Server::new(index));

    let use_socket_for_http_closure: SocketUser = Box::new({
        let server = server.clone();
        move |socket: TcpSocketWrapper| {
            let server = server.clone();
            Box::pin(async move {
                use_socket_for_http(&server, socket)
                    .await
                    .unwrap_or_else(|err| {
                        log::warn!("error: {err:?}");
                    })
            })
        }
    });

    let use_socket_for_https_closure: SocketUser = Box::new({
        let server = server.clone();
        let config = Arc::new(mk_config(cert_pem, priv_pem).unwrap());
        move |socket: TcpSocketWrapper| {
            let server = server.clone();
            let config = config.clone();
            Box::pin(async move {
                use_socket_for_https(&server, config, socket)
                    .await
                    .unwrap_or_else(|err| {
                        log::warn!("error: {err:?}");
                    })
            })
        }
    });

    for f in [use_socket_for_http_closure, use_socket_for_https_closure].map(Rc::new) {
        for _ in 0..NUM_SIMULTANEOUS_CONNECTIONS {
            spawner
                .spawn_local({
                    let network_ctx = network_ctx.clone();
                    let f = f.clone();
                    async move {
                        loop {
                            let socket = network_ctx.new_tcp_socket();
                            f(TcpSocketWrapper::new(socket)).await;
                        }
                    }
                })
                .unwrap()
        }
    }

    future::pending().await
}

async fn use_socket_for_http<'a, T: cpiofs::IO>(
    server: &'a Server<T>,
    mut socket: TcpSocketWrapper,
) -> Result<(), ClosedError<TcpSocketError>> {
    socket.inner_mut().accept(HTTP_PORT).await?;
    server.handle_connection(&mut socket).await?;
    socket.inner_mut().close().await?;
    Ok(())
}

async fn use_socket_for_https<'a, T: cpiofs::IO>(
    server: &'a Server<T>,
    config: Arc<mbedtls::ssl::Config>,
    mut socket: TcpSocketWrapper,
) -> Result<(), ClosedError<mbedtls::Error>> {
    socket.inner_mut().accept(HTTPS_PORT).await.unwrap(); // TODO
    let mut ctx = mbedtls::ssl::Context::new(config);
    ctx.establish_async(socket, None).await?;
    server.handle_connection(&mut ctx).await?;
    ctx.close_async().await?;
    let _ = ctx.take_io().unwrap().inner_mut().close().await; // TODO
    Ok(())
}

fn mk_config(cert_pem: &str, priv_pem: &str) -> mbedtls::Result<mbedtls::ssl::Config> {
    let entropy = Arc::new(insecure_dummy_rng());
    let rng = Arc::new(mbedtls::rng::CtrDrbg::new(entropy, None)?);
    let cert = Arc::new(mbedtls::x509::Certificate::from_pem_multiple(
        cert_pem.as_bytes(),
    )?);
    let key = Arc::new(mbedtls::pk::Pk::from_private_key(
        &mut insecure_dummy_rng(),
        priv_pem.as_bytes(),
        None,
    )?);
    let mut config = mbedtls::ssl::Config::new(
        mbedtls::ssl::config::Endpoint::Server,
        mbedtls::ssl::config::Transport::Stream,
        mbedtls::ssl::config::Preset::Default,
    );
    config.set_rng(rng);
    config.push_cert(cert, key)?;
    config.set_dbg_callback(
        DbgCallbackBuilder::default()
            .forward_log_level(log::Level::Warn)
            .build(),
    );
    Ok(config)
}
