// Copyright (c) 2026 Johan Mattsson
// License: MIT

#![doc = include_str!("../README.md")]

use crate::htmllexer::HtmlDynamicLexer;
use htmlcontentbuilder::HtmlContentBuilder;
use proc_macro::TokenStream;
use proc_macro::TokenTree;
use quote::{quote, quote_spanned};

mod htmlcontentbuilder;
mod htmllexer;
mod htmlquotes;
use proc_macro_error::proc_macro_error;

/// Declarative HTML macro.
///
/// Full documentation and examples are available
/// in the crate-level docs.
///
/// # Example for generating a String with HTML tags:
///
/// ```rust
/// use htmlmacro::html;
///
/// fn get_header() -> String {
///     let content = html! {{ <h1>Header</h1> }};
///     content
/// }
///
/// let html = html! {{
///     <div>
///         <include get_header() />
///         <ul>
///             <rust>
///                 for i in 1..3 {
///                     <markup>
///                         <li>List item $i</i>
///                     </markup>
///                 }
///             </rust>
///         </ul>
///     </div>
/// }};
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    let first = iter.next().expect("expected a group");

    let group = match first {
        TokenTree::Group(g) => g,
        _ => {
            let span = proc_macro2::Span::from(first.span());
            return quote_spanned! { span =>
                {
                    compile_error!(
                        "html!: does not use double brackets\n\
                        Hint: start the macro with double brackets containing your HTML body.\n\
                        Example: let code = html! {{ <div>Hello</div> }}\nNot let code = html! { <div>Hello</div> }"
                    );
                }
            }
            .into();
        }
    };

    let html_span = group.span();
    let line = html_span.line() as isize;
    let mut html_input = html_span.source_text().map(|cow| cow.to_string());

    if html_input.is_none() {
        html_input = Some(group.stream().to_string());
    }

    let src = match html_input {
        Some(s) => s,
        None => {
            return quote_spanned! { proc_macro2::Span::from(group.span()) =>
                { compile_error!("html!: Can't get html macro body as text."); }
            }
            .into();
        }
    };

    let html_tags = &src[1..src.len() - 1];
    let mut lexer = HtmlDynamicLexer::new(html_tags, line);
    lexer.lex();

    let mut content_builder = HtmlContentBuilder::new();
    let output = content_builder.process_tokens(lexer.tokens);

    let expanded = quote! {
        {
            #output
        }
    };

    return expanded.into();
}
