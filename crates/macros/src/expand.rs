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
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, spanned::Spanned, token::Brace, Ident, ItemFn, ReturnType, Stmt};

pub fn test(body: ItemFn, attrs: Attr) -> TokenStream {
    let name = &body.sig.ident;
    let inputs = &body.sig.inputs;
    let attr = &body.attrs;
    let ret = match body.sig.output {
        ReturnType::Default => quote!(),
        ReturnType::Type(ptr_tkn, ref ty) => quote!(#ptr_tkn #ty),
    };

    let body = &body.block;
    let router = &attrs.router;
    let setup = match attrs.setup {
        Some(path) => quote!(#path(&ctx).await;),
        None => quote!(),
    };

    let teardown = match attrs.teardown {
        Some(path) => quote!(#path(&ctx).await;),
        None => quote!(),
    };

    if !cfg!(feature = "testcontainers") && !attrs.containers.is_empty() {
        return syn::Error::new(
            body.span(),
            "`testcontainers` feature is not enabled and you passed in a list of containers to startup",
        )
        .into_compile_error();
    }

    let containers = attrs.containers.iter().map(|path| match path {
        crate::attr::PathOrExpr::Path(path) => quote!(ctx.containers_mut().push({
            let container = #path(&ctx).await;
            Box::new(container)
        });),

        crate::attr::PathOrExpr::Callable(callable) => quote!(ctx.containers_mut().push({
            let container = #callable;
            Box::new(container)
        });),
    });

    let serve = match attrs.router {
        Some(ref router) => quote!(ctx.serve(#router()).await;),
        None => quote!(),
    };

    quote! {
        #[::core::prelude::v1::test]
        #(#attr)*
        fn #name() #ret {
            async fn #name(#inputs) #ret {
                #body
            }

            let rt = ::tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create Tokio runtime?!");

            rt.block_on(async {
                // Create our TestContext
                let mut ctx = ::charted_testkit::TestContext::default();

                #setup
                #(#containers)*
                #serve

                let __fn_ptr: fn(_) -> _ = #name;
                let res = __fn_ptr(&ctx).await;

                #teardown
                res
            })
        }
    }
}
