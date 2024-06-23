<img align="right" width="auto" height="auto" alt="Noel (Trans Heart)" src="https://cdn.floofy.dev/images/trans.png" />

# 📦🦋 charted TestKit
charted **TestKit** is a testing library for [Axum](https://github.com/tokio-rs/axum), which extends [`libtest`] with its own [`test`] macro. The library can also be used for only Testcontainers as well.

**TestKit** was built to make integration testing easier for Axum services with Testcontainers support and additional macros to help build assertions based off [`Response`]s.

## Example
```rust
use charted_testkit::{test, TestContext, assert_successful, consume_body};
use axum::{body::Bytes, routing, Router};
use hyper::Method;

async fn hello() -> &'static str {
    "Hello, world!"
}

fn router() -> Router {
    Router::new().route("/", routing::get(hello))
}

#[test(router)]
async fn mytest(ctx: TestContext) {
    let res = ctx
        .request("/", Method::GET, None::<axum::body::Bytes>, |_| {})
        .await
        .expect("unable to send request");

    assert_successful!(res);

    let body = consume_body!(res);
    assert_eq!(body, Bytes::from_static(b"Hello, world!"));
}
```

## License
**TestKit** is released under the [`MIT` License](/LICENSE) with love and care by [Noelware, LLC.](https://noelware.org)! 🐻‍❄️🦋

Please read the `LICENSE` file in the [canonical repository](https://github.com/charted-dev/testkit) for more information on what you can do with the source code.

[`Response`]: https://docs.rs/http/latest/http/response/struct.Response.html
[`libtest`]: https://doc.rust-lang.org/stable/test
[`test`]: #
