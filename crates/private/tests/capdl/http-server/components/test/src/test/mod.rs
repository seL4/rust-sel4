use alloc::format;
use alloc::vec;
use core::str::pattern::Pattern;

use futures::prelude::*;
use futures::task::LocalSpawnExt;

use sel4_async_network::{SharedNetwork, TcpSocket};
use sel4_async_single_threaded_executor::LocalSpawner;

const PORT: u16 = 80;

const CONTENT_CPIO: &[u8] = include_bytes!(env!("CONTENT_CPIO"));

const NUM_SIMULTANEOUS_CONNECTIONS: usize = 1000;

pub async fn test(ctx: SharedNetwork, spawner: LocalSpawner) -> ! {
    let port = PORT;

    for _ in 0..NUM_SIMULTANEOUS_CONNECTIONS {
        spawner
            .spawn_local({
                let ctx = ctx.clone();
                async move {
                    loop {
                        let mut socket = ctx.new_tcp_socket();
                        socket.accept(port).await;
                        handle(&mut socket).await;
                        let r = socket.close().await;
                        if r.is_err() {
                            log::warn!("err");
                        }
                    }
                }
            })
            .unwrap()
    }

    future::pending().await
}

async fn handle(socket: &mut TcpSocket) {
    let mut buf = vec![0; 1024 * 16];
    let mut i = 0;
    loop {
        let n = socket.read(&mut buf[i..]).await;
        assert_ne!(n, 0);
        i += n;
        if request_complete(&buf) {
            break;
        }
    }
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);
    assert!(req.parse(&buf).unwrap().is_complete());
    log::info!("request: {:?}", req);
    serve(socket, req.path.unwrap()).await;
}

fn request_complete(buf: &[u8]) -> bool {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(buf).unwrap().is_complete()
}

async fn serve(socket: &mut TcpSocket, request_path: &str) {
    match find_file(request_path) {
        Some((name, content)) => {
            log::info!("serving file at {:?}", name);
            let content_type = content_type_from_name(name).unwrap();
            serve_file(socket, content_type, content).await;
            log::info!("done serving file at {:?}", name);
        }
        None => {
            log::info!("not found: {:?}", request_path);
            serve_not_found(socket).await;
        }
    }
}

async fn serve_file(socket: &mut TcpSocket, content_type: &str, content: &[u8]) {
    socket.write_all(b"HTTP/1.1 200 OK\r\n").await.unwrap();
    socket
        .write_all(format!("Content-Type: {}\r\n", content_type).as_bytes())
        .await
        .unwrap();
    socket
        .write_all(format!("Content-Length: {}\r\n", content.len()).as_bytes())
        .await
        .unwrap();
    socket.write_all(b"\r\n").await.unwrap();
    socket.write_all(content).await.unwrap();
}

async fn serve_not_found(socket: &mut TcpSocket) {
    let content = b"Not Found";
    socket
        .write_all(b"HTTP/1.1 404 Not Found\r\n")
        .await
        .unwrap();
    socket
        .write_all(b"Content-Type: text/plain; charset=utf-8\r\n")
        .await
        .unwrap();
    socket
        .write_all(format!("Content-Length: {}\r\n", content.len()).as_bytes())
        .await
        .unwrap();
    socket.write_all(b"\r\n").await.unwrap();
    socket.write_all(content).await.unwrap();
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

fn content_type_from_name(name: &str) -> Option<&'static str> {
    for (ext, ty) in MIME_ASSOCS {
        if ext.is_suffix_of(name) {
            return Some(ty);
        }
    }
    None
}

fn find_file(request_path: &str) -> Option<(&'static str, &'static [u8])> {
    for entry in cpio_reader::iter_files(CONTENT_CPIO) {
        if file_path_matches_request_path(entry.name(), request_path) {
            return Some((entry.name(), entry.file()));
        }
    }
    None
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
