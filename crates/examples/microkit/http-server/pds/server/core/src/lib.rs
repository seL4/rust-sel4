#![no_std]
#![feature(pattern)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;

use futures::future::{self, LocalBoxFuture};
use futures::task::LocalSpawnExt;

use mbedtls::ssl::async_io::ClosedError;

use sel4_async_block_io::{constant_block_sizes, BlockIO};
use sel4_async_block_io_fat as fat;
use sel4_async_network::{ManagedIface, TcpSocketError};
use sel4_async_network_mbedtls::{
    insecure_dummy_rng, mbedtls, seed_insecure_dummy_rng, DbgCallbackBuilder, TcpSocketWrapper,
};
use sel4_async_single_threaded_executor::LocalSpawner;
use sel4_async_timer_manager::TimerManager;

mod mime;
mod server;

use server::Server;

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

pub async fn run_server<T: BlockIO<BlockSize = constant_block_sizes::BlockSize512> + Clone>(
    _timers_ctx: TimerManager,
    network_ctx: ManagedIface,
    fs_block_io: T,
    spawner: LocalSpawner,
    cert_pem: &str,
    priv_pem: &str,
    max_num_simultaneous_connections: usize,
) -> ! {
    #[cfg(feature = "debug")]
    unsafe {
        mbedtls::set_global_debug_threshold(1);
    }

    seed_insecure_dummy_rng(0);

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

    let use_socket_for_https_closure: SocketUser<T> = Box::new({
        let config = Arc::new(mk_config(cert_pem, priv_pem).unwrap());
        move |server, socket| {
            let config = config.clone();
            Box::pin(async move {
                use_socket_for_https(server, config, socket)
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
                            f(server, TcpSocketWrapper::new(socket)).await;
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
        Server<fat::BlockIOWrapper<T>, fat::DummyTimeSource>,
        TcpSocketWrapper,
    ) -> LocalBoxFuture<'static, ()>,
>;

async fn use_socket_for_http<D: fat::BlockDevice + 'static, T: fat::TimeSource + 'static>(
    server: Server<D, T>,
    mut socket: TcpSocketWrapper,
) -> Result<(), ClosedError<TcpSocketError>> {
    socket.inner_mut().accept(HTTP_PORT).await?;
    server.handle_connection(&mut socket).await?;
    socket.inner_mut().close().await?;
    Ok(())
}

async fn use_socket_for_https<D: fat::BlockDevice + 'static, T: fat::TimeSource + 'static>(
    server: Server<D, T>,
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
