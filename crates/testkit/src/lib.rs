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

#![doc(html_logo_url = "https://cdn.floofy.dev/images/trans.png")]
#![doc = include_str!("../README.md")]

#[cfg(feature = "macros")]
pub use charted_testkit_macros::*;

mod macros;

use axum::{body::Bytes, extract::Request, Router};
use http_body_util::Full;
use hyper::{body::Incoming, Method};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client, ResponseFuture},
    rt::{TokioExecutor, TokioIo},
};
use std::{fmt::Debug, net::SocketAddr};
use tokio::{net::TcpListener, task::JoinHandle};
use tower::{Service, ServiceExt};

pub struct TestContext {
    _handle: Option<JoinHandle<()>>,
    client: Client<HttpConnector, http_body_util::Full<Bytes>>,
    http1: bool,
    addr: Option<SocketAddr>,

    // TODO(@auguwu): should `containers` be a `HashMap<TypeId, Box<dyn Any>>` to easily
    //                identify a image?
    #[cfg(feature = "testcontainers")]
    containers: Vec<Box<dyn ::std::any::Any + Send + Sync>>,

    #[cfg(feature = "http2")]
    http2: bool,
}

impl Debug for TestContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestContext").field("local_addr", &self.addr).finish()
    }
}

impl Default for TestContext {
    fn default() -> Self {
        TestContext {
            _handle: None,
            client: Client::builder(TokioExecutor::new()).build_http(),
            http1: true,
            addr: None,

            #[cfg(feature = "testcontainers")]
            containers: Vec::new(),

            #[cfg(feature = "http2")]
            http2: false,
        }
    }
}

impl TestContext {
    /// Allows HTTP/1 connections to be used. By disabling this, the ephermeral TCP listener
    /// won't know what to do unless HTTP/2 connections are allowed.
    pub fn allow_http1(mut self, yes: bool) -> Self {
        self.http1 = yes;
        self
    }

    /// Allows HTTP/2 connections to be used. By default, only HTTP/1 connections are allowed.
    #[cfg(feature = "http2")]
    pub fn allow_http2(mut self, yes: bool) -> Self {
        self.http2 = yes;
        self
    }

    /// Checks whenever if the ephermeral TCP listener should allow both HTTP/1 and HTTP/2 connections.
    #[cfg(feature = "http2")]
    pub fn allows_both(&self) -> bool {
        self.http1 && self.http2
    }

    /// Checks whenever if the ephermeral TCP listener should allow both HTTP/1 and HTTP/2 connections.
    #[cfg(not(feature = "http2"))]
    pub fn allows_both(&self) -> bool {
        self.http1
    }

    /// Returns a mutable [`Vec`] of allocated type-erased objects that should be [`ContainerAsync`].
    #[cfg(feature = "testcontainers")]
    pub fn containers_mut(&mut self) -> &mut Vec<Box<dyn ::std::any::Any + Send + Sync>> {
        &mut self.containers
    }

    /// Returns a [`ContainerAsync`] of a spawned container that can be accessed.
    #[cfg(feature = "testcontainers")]
    pub fn container<I: ::testcontainers::Image + 'static>(&self) -> Option<&::testcontainers::ContainerAsync<I>> {
        match self
            .containers
            .iter()
            .find(|x| x.is::<::testcontainers::ContainerAsync<I>>())
        {
            Some(container) => container.downcast_ref(),
            None => None,
        }
    }

    /// Returns a optional reference to a [socket address][SocketAddr] if [`TestContext::serve`] was called
    /// after this call.
    ///
    /// ## Example
    /// ```rust
    /// # use charted_testkit::TestContext;
    /// #
    /// let mut ctx = TestContext::default();
    /// assert!(ctx.server_addr().is_none());
    ///
    /// # // `IGNORE` is used since we don't actually want to spawn a server!
    /// # const IGNORE: &str = stringify! {
    /// ctx.serve(axum::Router::new()).await;
    /// assert!(ctx.server_addr().is_some());
    /// # };
    /// ```
    pub fn server_addr(&self) -> Option<&SocketAddr> {
        self.addr.as_ref()
    }

    /// Sends a request to the ephemeral server and returns a [`ResponseFuture`].
    ///
    /// ## Example
    /// ```rust,no_run
    /// # use charted_testkit::TestContext;
    /// # use axum::{routing, http::Method, body::Bytes};
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// async fn handler() -> &'static str {
    ///     "Hello, world!"
    /// }
    ///
    /// let mut ctx = TestContext::default();
    /// ctx.serve(axum::Router::new().route("/", routing::get(handler))).await;
    ///
    /// let res = ctx
    ///     .request::<_, Bytes, _>("/", Method::GET, None, |_| {})
    ///     .await
    ///     .expect("was unable to send request to ephermeral server");
    ///
    /// charted_testkit::assert_successful!(res);
    /// assert_eq!(charted_testkit::consume_body!(res), Bytes::from_static(b"Hello, world!"));
    /// # }
    /// ```
    pub fn request<U: AsRef<str> + 'static, B: Into<Bytes>, F: Fn(&mut Request<Full<Bytes>>)>(
        &self,
        uri: U,
        method: Method,
        body: Option<B>,
        build: F,
    ) -> ResponseFuture {
        let addr = self.server_addr().expect("failed to get socket address");

        let mut req = Request::<Full<Bytes>>::new(Full::new(body.map(Into::into).unwrap_or_default()));
        *req.method_mut() = method;
        *req.uri_mut() = format!("http://{addr}{}", uri.as_ref())
            .parse()
            .expect("failed to parse into `hyper::Uri`");

        build(&mut req);
        self.client.request(req)
    }

    /// Serves the ephermeral server.
    pub async fn serve(&mut self, router: Router) {
        if self._handle.is_some() {
            panic!("ephermeral server is already serving");
        }

        let allows_both = self.allows_both();
        let http1 = self.http1;

        #[cfg(feature = "http2")]
        let http2 = self.http2;

        #[cfg(not(feature = "http2"))]
        let http2 = false;

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to create tcp listener");

        self.addr = Some(listener.local_addr().expect("unable to get local addr"));

        // based off https://github.com/tokio-rs/axum/blob/934b1aac067dba596feb617817409345f9835db5/examples/serve-with-hyper/src/main.rs#L79-L118
        // since we don't need `axum::serve` and we want to customise the HTTP transport to use (i.e, if you want
        // to test HTTP/2 usage and not HTTP/1 usage)
        self._handle = Some(tokio::spawn(async move {
            let mut make_service = router.into_make_service_with_connect_info::<SocketAddr>();

            loop {
                let (socket, addr) = listener.accept().await.expect("failed to accept connection");
                let service = match make_service.call(addr).await {
                    Ok(service) => service,
                    Err(e) => match e {},
                };

                tokio::spawn(async move {
                    let socket = TokioIo::new(socket);
                    let hyper_service =
                        hyper::service::service_fn(move |request: Request<Incoming>| service.clone().oneshot(request));

                    if allows_both {
                        #[cfg(feature = "http2")]
                        if let Err(err) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                            .serve_connection_with_upgrades(socket, hyper_service)
                            .await
                        {
                            eprintln!("failed to serve connection: {err:#}");
                        }

                        #[cfg(not(feature = "http2"))]
                        if let Err(err) = hyper::server::conn::http1::Builder::new()
                            .serve_connection(socket, hyper_service)
                            .await
                        {
                            eprintln!("failed to serve HTTP/1 connection: {err:#}");
                        }
                    } else if http2 {
                        #[cfg(feature = "http2")]
                        if let Err(err) = hyper::server::conn::http2::Builder::new(TokioExecutor::new())
                            .serve_connection(socket, hyper_service)
                            .await
                        {
                            eprintln!("failed to serve HTTP/2 connection: {err:#}");
                        }
                    } else if http1 {
                        if let Err(err) = hyper::server::conn::http1::Builder::new()
                            .serve_connection(socket, hyper_service)
                            .await
                        {
                            eprintln!("failed to serve HTTP/1 connection: {err:#}");
                        }
                    } else {
                        panic!("unable to serve connection due to no HTTP stream to process");
                    }
                });
            }
        }));
    }
}

// Private APIs used by macros; do not use!
#[doc(hidden)]
pub mod __private {
    pub use http_body_util::BodyExt;
}

#[cfg(test)]
mod tests {
    use crate::{assert_successful, consume_body, TestContext};
    use axum::{body::Bytes, routing, Router};
    use hyper::Method;

    async fn hello() -> &'static str {
        "Hello, world!"
    }

    fn router() -> Router {
        Router::new().route("/", routing::get(hello))
    }

    #[tokio::test]
    #[cfg_attr(
        windows,
        ignore = "fails on Windows: hyper_util::client::legacy::Error(Connect, ConnectError(\"tcp connect error\", Os { code: 10049, kind: AddrNotAvailable, message: \"The requested address is not valid in its context.\" })))"
    )]
    async fn test_usage() {
        let mut ctx = TestContext::default();
        ctx.serve(router()).await;

        let res = ctx
            .request("/", Method::GET, None::<axum::body::Bytes>, |_| {})
            .await
            .expect("unable to send request");

        assert_successful!(res);
        assert_eq!(consume_body!(res), Bytes::from_static(b"Hello, world!"));
    }

    #[cfg(feature = "testcontainers")]
    #[tokio::test]
    #[cfg_attr(
        not(target_os = "linux"),
        ignore = "this will only probably work on Linux (requires a working Docker daemon)"
    )]
    async fn test_testcontainers_in_ctx() {
        use testcontainers::runners::AsyncRunner;

        let mut ctx = TestContext::default();
        let valkey = ::testcontainers::GenericImage::new("valkey/valkey", "7.2.6")
            .start()
            .await
            .expect("failed to start valkey image");

        ctx.containers_mut().push(Box::new(valkey));
        assert!(ctx.container::<::testcontainers::GenericImage>().is_some());
    }
}
