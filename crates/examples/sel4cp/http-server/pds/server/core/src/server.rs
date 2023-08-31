use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use core::str::pattern::Pattern;

use mbedtls::ssl::async_io::{AsyncIo, AsyncIoExt, ClosedError};

use sel4_async_block_io::BytesIO;
use sel4_async_block_io_cpiofs as cpiofs;
use sel4_async_network_mbedtls::mbedtls;

pub(crate) struct Server<T> {
    index: cpiofs::Index<T>,
}

impl<T: BytesIO> Server<T> {
    pub(crate) fn new(index: cpiofs::Index<T>) -> Self {
        Self { index }
    }

    pub(crate) async fn handle_connection<U: AsyncIo>(
        &self,
        conn: &mut U,
    ) -> Result<(), ClosedError<U::Error>> {
        loop {
            let mut buf = vec![0; 1024 * 16];
            let mut i = 0;
            loop {
                let n = conn.recv(&mut buf[i..]).await?;
                assert_ne!(n, 0);
                i += n;
                if is_request_complete(&buf[..i]).unwrap_or(false) {
                    break;
                }
            }
            let mut headers = [httparse::EMPTY_HEADER; 32];
            let mut req = httparse::Request::new(&mut headers);
            let mut keep_alive = false;
            match req.parse(&buf) {
                Ok(status) => {
                    assert!(status.is_complete());
                    self.handle_request(conn, req.path.unwrap()).await?;
                    if should_keep_alive(&req) {
                        keep_alive = true;
                    }
                }
                Err(err) => {
                    log::warn!("error parsing request: {err:?}");
                }
            }
            if !keep_alive {
                break;
            }
        }
        Ok(())
    }

    async fn handle_request<U: AsyncIo>(
        &self,
        conn: &mut U,
        request_path: &str,
    ) -> Result<(), ClosedError<U::Error>> {
        match self.lookup_request_path(request_path).await {
            RequestPathStatus::Ok { file_path, entry } => {
                let content_type = content_type_from_name(&file_path);
                self.serve_file(conn, content_type, &entry).await?;
            }
            RequestPathStatus::MovedPermanently { location } => {
                self.serve_moved_permanently(conn, &location).await?;
            }
            RequestPathStatus::NotFound => {
                self.serve_not_found(conn).await?;
            }
        }
        Ok(())
    }

    async fn serve_file<U: AsyncIo>(
        &self,
        conn: &mut U,
        content_type: &str,
        entry: &cpiofs::Entry,
    ) -> Result<(), ClosedError<U::Error>> {
        self.start_response_headers(conn, 200, "OK").await?;
        self.send_response_header(conn, "Content-Type", content_type.as_bytes())
            .await?;
        self.send_response_header(
            conn,
            "Content-Length",
            entry.data_size().to_string().as_bytes(),
        )
        .await?;
        self.finish_response_headers(conn).await?;
        {
            let mut buf = vec![0; 2048];
            let mut pos = 0;
            while pos < entry.data_size() {
                let n = buf.len().min(entry.data_size() - pos);
                self.index.read_data(entry, pos, &mut buf[..n]).await;
                conn.send_all(&buf[..n]).await?;
                pos += n;
            }
        }
        Ok(())
    }

    async fn serve_moved_permanently<U: AsyncIo>(
        &self,
        conn: &mut U,
        location: &str,
    ) -> Result<(), ClosedError<U::Error>> {
        let phrase = "Moved Permanently";
        self.start_response_headers(conn, 301, phrase).await?;
        self.send_response_header(conn, "Content-Type", b"text/plain")
            .await?;
        self.send_response_header(conn, "Content-Length", phrase.len().to_string().as_bytes())
            .await?;
        self.send_response_header(conn, "Location", location.as_bytes())
            .await?;
        self.finish_response_headers(conn).await?;
        conn.send_all(phrase.as_bytes()).await?;
        Ok(())
    }

    async fn serve_not_found<U: AsyncIo>(&self, conn: &mut U) -> Result<(), ClosedError<U::Error>> {
        let phrase = "Not Found";
        self.start_response_headers(conn, 404, phrase).await?;
        self.send_response_header(conn, "Content-Type", b"text/plain")
            .await?;
        self.send_response_header(conn, "Content-Length", phrase.len().to_string().as_bytes())
            .await?;
        self.finish_response_headers(conn).await?;
        conn.send_all(phrase.as_bytes()).await?;
        Ok(())
    }

    async fn start_response_headers<U: AsyncIo>(
        &self,
        conn: &mut U,
        status_code: usize,
        reason_phrase: &str,
    ) -> Result<(), ClosedError<U::Error>> {
        conn.send_all(b"HTTP/1.1 ").await?;
        conn.send_all(status_code.to_string().as_bytes()).await?;
        conn.send_all(b" ").await?;
        conn.send_all(reason_phrase.as_bytes()).await?;
        conn.send_all(b"\r\n").await?;
        Ok(())
    }

    async fn send_response_header<U: AsyncIo>(
        &self,
        conn: &mut U,
        name: &str,
        value: &[u8],
    ) -> Result<(), ClosedError<U::Error>> {
        conn.send_all(name.as_bytes()).await?;
        conn.send_all(b": ").await?;
        conn.send_all(value).await?;
        conn.send_all(b"\r\n").await?;
        Ok(())
    }

    async fn finish_response_headers<U: AsyncIo>(
        &self,
        conn: &mut U,
    ) -> Result<(), ClosedError<U::Error>> {
        conn.send_all(b"\r\n").await?;
        Ok(())
    }

    async fn lookup_request_path(&self, request_path: &str) -> RequestPathStatus {
        if !"/".is_prefix_of(request_path) {
            return RequestPathStatus::NotFound;
        }
        let has_trailing_slash = "/".is_suffix_of(request_path);
        let normalized = request_path.trim_matches('/');
        if normalized.is_empty() {
            let file_path = "index.html";
            if let Some(location) = self.index.lookup(file_path) {
                let entry = self.index.read_entry(location).await;
                if entry.ty() == cpiofs::EntryType::RegularFile {
                    return RequestPathStatus::Ok {
                        file_path: file_path.to_owned(),
                        entry,
                    };
                }
            }
        } else if let Some(location) = self.index.lookup(normalized) {
            let entry = self.index.read_entry(location).await;
            match entry.ty() {
                cpiofs::EntryType::RegularFile => {
                    return RequestPathStatus::Ok {
                        file_path: normalized.to_owned(),
                        entry,
                    };
                }
                cpiofs::EntryType::Directory => {
                    if !has_trailing_slash {
                        return RequestPathStatus::MovedPermanently {
                            location: format!("{}/", request_path),
                        };
                    }
                    let normalized_with_index_html = format!("{}/index.html", normalized);
                    if let Some(location) = self.index.lookup(&normalized_with_index_html) {
                        let entry = self.index.read_entry(location).await;
                        return RequestPathStatus::Ok {
                            file_path: normalized_with_index_html,
                            entry,
                        };
                    }
                }
                // TODO handle symlinks
                _ => {}
            }
        }
        RequestPathStatus::NotFound
    }
}

#[derive(Debug)]
enum RequestPathStatus {
    Ok {
        file_path: String,
        entry: cpiofs::Entry,
    },
    MovedPermanently {
        location: String,
    },
    NotFound,
}

fn is_request_complete(buf: &[u8]) -> Result<bool, httparse::Error> {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(buf).map(|status| status.is_complete())
}

fn should_keep_alive(req: &httparse::Request) -> bool {
    let version = req.version.unwrap();
    let default = match version {
        0 => false,
        1 => true,
        _ => panic!(),
    };
    for header in req.headers.iter() {
        if header.name.to_lowercase() == "Connection" {
            if header.value == b"close" {
                return false;
            }
            if header.value == b"keep-alive" {
                return true;
            }
            panic!();
        }
    }
    default
}

fn content_type_from_name(name: &str) -> &'static str {
    for (ext, ty) in MIME_ASSOCS {
        if ext.is_suffix_of(name) {
            return ty;
        }
    }
    DEFAULT_MIME_TYPE
}

const DEFAULT_MIME_TYPE: &str = "application/octet-stream";

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
