//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

pub(crate) fn content_type_from_name(name: &str) -> &'static str {
    for (ext, ty) in MIME_ASSOCS {
        if name.ends_with(ext) {
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
