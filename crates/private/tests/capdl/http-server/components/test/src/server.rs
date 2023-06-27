use alloc::format;
use alloc::rc::Rc;
use alloc::vec;
use core::str::pattern::Pattern;

use futures::prelude::*;
use futures::task::LocalSpawnExt;

use sel4_async_network::{SharedNetwork, TcpSocket, TcpSocketError};
use sel4_async_single_threaded_executor::LocalSpawner;

use crate::{CpioEntry, CpioIOImpl, CpioIndex};

const PORT: u16 = 80;

const NUM_SIMULTANEOUS_CONNECTIONS: usize = 1000;

pub async fn run_server(ctx: SharedNetwork, blk_device: CpioIOImpl, spawner: LocalSpawner) -> ! {
    let ix = CpioIndex::create(blk_device).await;

    let server = Server { ix: Rc::new(ix) };

    for _ in 0..NUM_SIMULTANEOUS_CONNECTIONS {
        let ctx = ctx.clone();
        let server = server.clone();
        spawner
            .spawn_local(async move {
                loop {
                    let socket = ctx.new_tcp_socket();
                    if let Err(err) = server.use_socket(socket).await {
                        log::warn!("err: {:?}", err);
                    }
                }
            })
            .unwrap()
    }

    future::pending().await
}

#[derive(Clone)]
struct Server {
    ix: Rc<CpioIndex<CpioIOImpl>>,
}

impl Server {
    async fn use_socket(&self, mut socket: TcpSocket) -> Result<(), TcpSocketError> {
        let port = PORT;
        socket.accept(port).await?;
        self.handle_connection(&mut socket).await?;
        socket.close().await?;
        Ok(())
    }

    async fn handle_connection(&self, socket: &mut TcpSocket) -> Result<(), TcpSocketError> {
        let mut buf = vec![0; 1024 * 16];
        let mut i = 0;
        loop {
            let n = socket.recv(&mut buf[i..]).await?;
            assert_ne!(n, 0);
            i += n;
            if is_request_complete(&buf) {
                break;
            }
        }
        let mut headers = [httparse::EMPTY_HEADER; 32];
        let mut req = httparse::Request::new(&mut headers);
        assert!(req.parse(&buf).unwrap().is_complete());
        log::info!("request: {:?}", req);
        self.handle_request(socket, req.path.unwrap()).await?;
        Ok(())
    }

    async fn handle_request(
        &self,
        socket: &mut TcpSocket,
        request_path: &str,
    ) -> Result<(), TcpSocketError> {
        match self.find_file(request_path) {
            Some((name, entry)) => {
                log::info!("serving file at {:?}", name);
                let content_type = content_type_from_name(name).unwrap();
                self.serve_file(socket, content_type, entry).await?;
                log::info!("done serving file at {:?}", name);
            }
            None => {
                log::info!("not found: {:?}", request_path);
                self.serve_not_found(socket).await?;
            }
        }
        Ok(())
    }

    async fn serve_file(
        &self,
        socket: &mut TcpSocket,
        content_type: &str,
        entry: &CpioEntry,
    ) -> Result<(), TcpSocketError> {
        socket.send(b"HTTP/1.1 200 OK\r\n").await?;
        socket
            .send(format!("Content-Type: {}\r\n", content_type).as_bytes())
            .await?;
        socket
            .send(format!("Content-Length: {}\r\n", entry.data_size()).as_bytes())
            .await?;
        socket.send(b"\r\n").await?;
        {
            let mut buf = vec![0; 2048];
            let mut pos = 0;
            while pos < entry.data_size() {
                let n = buf.len().min(entry.data_size() - pos);
                self.ix.read(entry, pos, &mut buf[..n]).await;
                socket.send(&buf[..n]).await?;
                pos += n;
            }
        }
        Ok(())
    }

    async fn serve_not_found(&self, socket: &mut TcpSocket) -> Result<(), TcpSocketError> {
        let content = b"Not Found";
        socket.send(b"HTTP/1.1 404 Not Found\r\n").await?;
        socket
            .send(b"Content-Type: text/plain; charset=utf-8\r\n")
            .await?;
        socket
            .send(format!("Content-Length: {}\r\n", content.len()).as_bytes())
            .await?;
        socket.send(b"\r\n").await?;
        socket.send(content).await?;
        Ok(())
    }

    fn find_file(&self, request_path: &str) -> Option<(&str, &CpioEntry)> {
        for (entry_path, entry) in self.ix.entries().iter() {
            if file_path_matches_request_path(entry_path, request_path) {
                return Some((entry_path, entry));
            }
        }
        None
    }
}

fn is_request_complete(buf: &[u8]) -> bool {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(buf).unwrap().is_complete()
}

fn file_path_matches_request_path(file_path: &str, request_path: &str) -> bool {
    let path_after_leading_slash = request_path.strip_prefix("/").unwrap();
    if path_after_leading_slash == "" && file_path == "index.html" {
        return true;
    }
    if let Some(entry_name_before_slash_index_html) = file_path.strip_suffix("/index.html") {
        if let Some(path_after_leading_slash_and_before_trailing_slash) =
            path_after_leading_slash.strip_suffix("/")
        {
            if entry_name_before_slash_index_html
                == path_after_leading_slash_and_before_trailing_slash
            {
                return true;
            }
        }
    }
    if file_path == path_after_leading_slash {
        return true;
    };
    false
}

fn content_type_from_name(name: &str) -> Option<&'static str> {
    for (ext, ty) in MIME_ASSOCS {
        if ext.is_suffix_of(name) {
            return Some(ty);
        }
    }
    None
}

const MIME_ASSOCS: &[(&str, &str)] = &[
    (".css", "text/css"),
    (".html", "text/html; charset=utf-8"),
    (".ico", "image/vnd.microsoft.icon"),
    (".jpg", "image/jpeg"),
    (".js", "text/javascript; charset=utf-8"),
    (".mp4", "video/mp4"),
    (".pdf", "application/pdf"),
    (".png", "image/png"),
    (".svg", "image/svg+xml"),
    (".ttf", "font/ttf"),
    (".txt", "text/plain; charset=utf-8"),
    (".woff", "font/woff"),
    (".woff2", "font/woff2"),
    (".zip", "application/zip"),
];
