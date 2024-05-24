// 📦🦋 charted TestKit: testing library for Axum services with testcontainers support
// Copyright (c) 2024 Noelware, LLC. <team@noelware.org>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// Checks whenever if a [`Response`][axum::http::response::Response] is successful
/// or not.
///
/// ## Example
/// ```rust
/// # use axum::http::{response::Response, StatusCode};
/// #
/// let res = Response::builder().status(StatusCode::OK).body(()).expect("response to be avaliable");
/// charted_testkit::assert_successful!(res);
/// ```
#[macro_export]
macro_rules! assert_successful {
    ($res:expr) => {
        assert!(($res).status().is_success());
    };
}

/// Macro to easily assert if a given [response][axum::http::response::Response]'s status code
/// is the same as one you provide.
///
/// ## Example
/// ```rust
/// # use axum::http::{response::Response, StatusCode};
/// #
/// let res = Response::builder().status(StatusCode::OK).body(()).expect("response to be avaliable");
/// charted_testkit::assert_status_code!(res, StatusCode::OK);
/// ```
#[macro_export]
macro_rules! assert_status_code {
    ($res:expr, $status:expr) => {
        assert_eq!($status, ($res).status());
    };
}

/// Macro to consume the full body of a [response][axum::http::response::Response] and returns
/// a [`Bytes`][axum::body::Bytes] container.
///
/// ```rust
/// # use axum::{http::response::Response, http::StatusCode, body::Bytes};
/// #
/// # #[tokio::main]
/// # async fn main() {
/// let res = Response::builder()
///     .status(StatusCode::OK)
///     .body(String::from("Hello, world!"))
///     .expect("response to be constructed");
///
/// assert_eq!(charted_testkit::consume_body!(res), Bytes::from_static(b"Hello, world!"));
/// # }
/// ```
#[macro_export]
macro_rules! consume_body {
    ($res:expr) => {{
        use ::http_body_util::BodyExt;

        let collected = ($res).collect().await.expect("failed to consume full body");
        collected.to_bytes()
    }};
}
