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

use proc_macro2::Span;

use syn::ExprCall;
#[allow(unused_imports)]
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned as _,
    Expr, ExprLit, ExprPath, Ident, Lit, Path, PathSegment, Result, Token,
};

macro_rules! err {
    ($span:expr, $msg:literal) => {{
        #[allow(unused_imports)]
        use syn::spanned::Spanned;

        ::syn::Error::new($span, $msg)
    }};

    ($span:expr, $msg:expr) => {{
        use syn::spanned::Spanned;
        ::syn::Error::new($span, $msg)
    }};

    ($msg:literal) => {
        ::syn::Error::new(::proc_macro2::Span::call_site(), $msg)
    };

    ($msg:expr) => {
        ::syn::Error::new(::proc_macro2::Span::call_site(), $msg)
    };
}

mod kw {
    syn::custom_keyword!(containers);
    syn::custom_keyword!(teardown);
    syn::custom_keyword!(router);
    syn::custom_keyword!(setup);
}

pub(crate) enum PathOrExpr {
    Path(Path),
    Callable(ExprCall),
}

#[derive(Default)]
pub struct Attr {
    pub containers: Vec<PathOrExpr>,
    pub teardown: Option<Path>,
    pub router: Option<Path>,
    pub setup: Option<Path>,
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut me = Attr::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::containers) {
                // containers = ["(litstr)" | path_to_fn]
                input.parse::<kw::containers>()?;
                input.parse::<Token![=]>()?;

                let content;
                bracketed!(content in input);

                let mut paths = Vec::new();

                for expr in Punctuated::<Expr, Token![,]>::parse_terminated(&content)? {
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
                        let ident = Ident::new(&s.value(), s.span());
                        paths.push(PathOrExpr::Path((PathSegment::from(ident)).into()));
                    } else if let Expr::Path(ExprPath { path, .. }) = expr {
                        paths.push(PathOrExpr::Path(path));
                    } else if let Expr::Call(callable) = expr {
                        paths.push(PathOrExpr::Callable(callable));
                    } else {
                        return Err(err!(
                            expr.span(),
                            "expected a literal string, valid path to a function, or a function call"
                        ));
                    }
                }

                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }

                me.containers = paths;
                continue;
            } else if lookahead.peek(kw::teardown) {
                if me.teardown.is_some() {
                    return Err(err!("cannot overwrite an existing teardown function"));
                }

                // teardown
                // teardown = "path"
                // teardown = module::to::teardown
                input.parse::<kw::teardown>()?;

                if !input.peek(Token![=]) {
                    me.teardown = Some(PathSegment::from(Ident::new("teardown", Span::call_site())).into());
                    comma_if_not_empty(input)?;

                    continue;
                }

                input.parse::<Token![=]>()?;

                me.teardown = Some(parse_literal_or_path(input)?);
                comma_if_not_empty(input)?;

                continue;
            } else if lookahead.peek(kw::setup) {
                if me.setup.is_some() {
                    return Err(err!("cannot overwrite an existing setup function"));
                }

                // setup
                // setup = "path"
                // setup = module::to::teardown
                input.parse::<kw::setup>()?;

                if !input.peek(Token![=]) {
                    me.setup = Some(PathSegment::from(Ident::new("setup", Span::call_site())).into());
                    comma_if_not_empty(input)?;

                    continue;
                }

                input.parse::<Token![=]>()?;

                me.setup = Some(parse_literal_or_path(input)?);
                comma_if_not_empty(input)?;

                continue;
            } else if lookahead.peek(kw::router) {
                if me.router.is_some() {
                    return Err(err!(Span::call_site(), "router is already defined"));
                }

                // router
                // router = "path_to_router"
                // router = path_to_router_also
                input.parse::<kw::router>()?;
                if !input.peek(Token![=]) {
                    me.router = Some(router_path());

                    comma_if_not_empty(input)?;
                    continue;
                }

                me.router = Some(parse_literal_or_path(input)?);
                comma_if_not_empty(input)?;

                continue;
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(me)
    }
}

fn router_path() -> Path {
    PathSegment::from(Ident::new("router", Span::call_site())).into()
}

fn parse_literal_or_path(input: ParseStream) -> Result<Path> {
    let expr = input.parse::<Expr>()?;
    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
        Ok(PathSegment::from(Ident::new(&s.value(), s.span())).into())
    } else if let Expr::Path(ExprPath { path, .. }) = expr {
        Ok(path)
    } else {
        return Err(err!(
            expr.span(),
            "expected a literal string or valid path to a function for a teardown function"
        ));
    }
}

fn comma_if_not_empty(input: ParseStream) -> Result<()> {
    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok(())
}
