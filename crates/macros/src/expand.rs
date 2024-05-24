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

#![allow(unused)]

use crate::attr::Attr;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{token::Brace, ItemFn, Stmt};

struct Body<'a> {
    brace_token: Brace,
    stmts: &'a [Stmt],
}

impl ToTokens for Body<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brace_token.surround(tokens, |tt| {
            for stmt in self.stmts {
                stmt.to_tokens(tt);
            }
        });
    }
}

pub fn test(mut body: ItemFn, attrs: Attr) -> proc_macro2::TokenStream {
    body.sig.asyncness = None;

    let name = &body.sig.ident;

    // Build a Tokio runtime in vein of `#[tokio::test]`
    let rt = quote! {
        ::tokio::runtime::Builder::new_multi_thread()
    };

    let header = quote!(#[::core::prelude::v1::test]);
    let endgoal = quote! {
        #rt.enable_all().build().expect("failed to build runtime")
    };

    quote! {}
}
