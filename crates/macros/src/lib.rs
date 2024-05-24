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

mod attr;
mod expand;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

/// Represents a procedural attribute macro that does the heavy lifting of `charted_testkit` for you. This macro
/// also manages:
///
/// * setup functions, where a `fn(&TestContext) -> Result<(), Box<dyn ::std::error::Error>>` is called on each
///   test to set it up
/// * teardown functions, where a `fn(&TestContext) -> Result<(), Box<dyn ::std::error::Error>>` is called when
///   a test is done being executed
/// * list of testcontainers that will be spawned if a Docker environment is avaliable.
#[proc_macro_attribute]
pub fn test(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let body = parse_macro_input!(item as ItemFn);
    let attrs = match syn::parse::<attr::Attr>(attrs) {
        Ok(attrs) => attrs,
        Err(e) => return e.into_compile_error().into(),
    };

    expand::test(body, attrs).into()
}
