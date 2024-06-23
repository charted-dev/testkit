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

use axum::{body::Bytes, http::Method};
use charted_testkit::{assert_successful, consume_body, TestContext};
use charted_testkit_macros::test;

async fn setup(_ctx: &TestContext) {
    println!("in setup function!!!");
}

async fn hello() -> &'static str {
    "Hello, world?"
}

fn router() -> axum::Router {
    axum::Router::new().route("/", axum::routing::get(hello))
}

#[test(setup, router)]
#[cfg_attr(
    windows,
    ignore = "fails on Windows: hyper_util::client::legacy::Error(Connect, ConnectError(\"tcp connect error\", Os { code: 10049, kind: AddrNotAvailable, message: \"The requested address is not valid in its context.\" })))"
)]
async fn usage(ctx: &TestContext) {
    let res = ctx
        .request("/", Method::GET, None::<axum::body::Bytes>, |_| {})
        .await
        .expect("unable to send request");

    assert_successful!(res);

    let body = consume_body!(res);
    assert_eq!(body, Bytes::from_static(b"Hello, world?"));
}
