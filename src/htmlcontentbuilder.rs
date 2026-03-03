// Copyright (c) 2026 Johan Mattsson
// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// See LICENSE-APACHE or LICENSE-MIT for details.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse_str;

use crate::htmllexer::*;
use crate::htmlquotes::html_quotes;

use proc_macro::Span;
use proc_macro_error::abort;

#[derive(Debug, Clone)]
pub enum ErrType {
    NoIncludeEndSlash,
    IncludeOutsideOfMarkup,
    BadTag,
    VarExpr,
    VarSingleExpr,
    Other,
}

#[derive(Debug, Clone)]
pub struct HtmlError {
    pub message: String,
    pub hint: String,
    pub line: isize,
    pub err_type: ErrType,
}

impl HtmlError {
    fn abort(&self) {
        let mut compile_message = String::new();

        compile_message.push_str("HTML macro error. ");

        if self.line >= 0 {
            let line = format!("Line: {}. ", self.line);
            compile_message.push_str(&line);
        }

        if !self.message.is_empty() {
            compile_message.push_str(&self.message);
            compile_message.push('\n');
        }

        if !self.hint.is_empty() {
            let hint = format!("Hint: {}\n", self.hint);
            compile_message.push_str(&hint);
        }

        let err_type = format!("Type: {:?}", self.err_type);
        compile_message.push_str(&err_type);

        abort!(Span::call_site(), "{}", compile_message);
    }
}

pub struct HtmlContentBuilder {
    is_markup: bool,
    is_include_tag: bool,
    current_text: String,
    first_text_line: isize,
    rust_node_count: u32,
    rust_buffer: String,
}

impl HtmlContentBuilder {
    pub fn new() -> HtmlContentBuilder {
        HtmlContentBuilder {
            is_markup: true,
            is_include_tag: false,
            current_text: String::new(),
            first_text_line: -1,
            rust_node_count: 0,
            rust_buffer: String::new(),
        }
    }

    fn make_text_node(&mut self) -> Result<(), HtmlError> {
        if self.current_text != "" {
            let text = &self.current_text;
            let line = self.first_text_line;
            let rust_push_to_html_string = html_quotes(text.as_str(), line)?;
            self.rust_buffer.push_str(&rust_push_to_html_string);
            self.current_text.clear();
        }

        Ok(())
    }

    pub fn process_tokens(&mut self, input: Vec<HtmlToken>) -> TokenStream2 {
        let result = self.process_tokens_string(input);

        match result {
            Ok(_) => {}
            Err(e) => {
                e.abort();
            }
        }

        let result = self.rust_buffer.clone();
        let mut rust_code = match result.parse::<TokenStream2>() {
            Ok(ts) => ts,
            Err(e) => {
                let err = format!(
                    "Invalid generated rust code from html! macro: '{}'. parse error: {}",
                    result, e
                );
                let err_ts = quote! { compile_error!(#err); };
                TokenStream2::from(err_ts)
            }
        };

        let code = quote! { __html_macro_content };
        rust_code.extend(TokenStream2::from(code));
        rust_code
    }

    fn process_tokens_string(&mut self, input: Vec<HtmlToken>) -> Result<(), HtmlError> {
        self.rust_buffer
            .push_str("let mut __html_macro_content = String::new();\n");

        let tokens: Vec<HtmlToken> = input;
        let length = tokens.len();
        let mut current_index = 0;

        while current_index < length {
            let token = &tokens[current_index];

            if next_is_include_tag(&tokens, current_index) {
                if !self.is_markup {
                    return Err(HtmlError {
                        message: "Include tag outside of markup.".into(),
                        hint: String::new(),
                        line: token.line_number,
                        err_type: ErrType::IncludeOutsideOfMarkup,
                    });
                }

                self.make_text_node()?;
                self.is_include_tag = true;
                current_index = self.process_include(&tokens, current_index)?;
                continue;
            }

            if next_is_rust_start(&tokens, current_index) {
                self.is_markup = false;
                self.rust_node_count += 1;
                current_index += 3; // skip <rust>
                self.make_text_node()?;
                continue;
            }

            if next_is_rust_end(&tokens, current_index) {
                if self.rust_node_count == 0 {
                    return Err(HtmlError {
                        message: "</rust> tag seen but no starting <rust> tag found".into(),
                        hint: String::new(),
                        line: token.line_number,
                        err_type: ErrType::BadTag,
                    });
                }

                self.is_markup = true;
                current_index += 4; // skip </rust>
                self.rust_node_count -= 1;
                self.make_text_node()?;
                continue;
            }

            if next_is_markup_start(&tokens, current_index) && !self.is_markup {
                self.is_markup = true;
                current_index += 3; // skip <markup>
                self.make_text_node()?;
                continue;
            }

            if self.next_is_markup_end(&tokens, current_index) {
                self.is_markup = false;
                current_index += 4; // skip </markup>
                self.make_text_node()?;
                continue;
            }

            // append token to stream
            let regular_token = token;

            if self.is_markup {
                self.add_token_to_text(regular_token);
            } else {
                let s = regular_token.to_string();
                self.rust_buffer.push_str(&s);
            }

            current_index += 1;
        }

        self.make_text_node()?;
        Ok(())
    }

    fn process_include(
        &mut self,
        tokens: &Vec<HtmlToken>,
        current_index: usize,
    ) -> Result<usize, HtmlError> {
        let mut current_index = current_index;
        let mut include_code_string = String::new();
        let mut rust = String::new();
        let length = tokens.len();
        let start_line = (&tokens[current_index]).line_number;

        current_index += 2;

        loop {
            let token = &tokens[current_index];
            let next = peek(&tokens, current_index, 1);

            if current_index >= length || next.is_none() {
                return Err(HtmlError {
                    message: "No end slash for include tag found.".into(),
                    hint: "<include example() />".into(),
                    line: start_line,
                    err_type: ErrType::NoIncludeEndSlash,
                });
            }

            if token.value == "/" && next.unwrap().value == ">" {
                self.is_include_tag = false;
                current_index += 2;

                let inc_code = include_code_string.trim();

                if inc_code.is_empty() {
                    return Err(HtmlError {
                        message: "Empty include expression.".into(),
                        hint: "<include example() />".into(),
                        line: start_line,
                        err_type: ErrType::Other,
                    });
                }

                match parse_str::<syn::Expr>(&inc_code) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err(HtmlError {
                            message: format!("Invalid include expression: {}", inc_code),
                            hint: "<include example() />".into(),
                            line: start_line,
                            err_type: ErrType::Other,
                        });
                    }
                }

                rust.push_str("let __included_html = ");
                rust.push_str(inc_code);
                rust.push_str(";\n");

                rust.push_str("__html_macro_content.push_str(__included_html.as_str());\n");

                self.rust_buffer.push_str(rust.as_str());
                break;
            }

            include_code_string.push_str(token.to_string().as_str());
            current_index += 1;
        }

        return Ok(current_index);
    }

    fn add_token_to_text(&mut self, token: &HtmlToken) {
        let text = token.to_string();
        self.push_str(&text, token.line_number);
    }

    fn push_str(&mut self, token: &str, line_number: isize) {
        if self.current_text.is_empty() {
            self.first_text_line = line_number;
        }

        self.current_text.push_str(token);

        if self.current_text.ends_with("<rust>") {
            // just in case there is a bug
            panic!("{:?} ends with start tag.", self.current_text);
        }

        if self.current_text.ends_with("</rust>") {
            panic!("{:?} ends with end tag.", self.current_text);
        }
    }

    fn next_is_markup_end(&self, tokens: &Vec<HtmlToken>, i: usize) -> bool {
        return is_tag_end("markup", tokens, i);
    }
}

fn next_is_rust_start(tokens: &Vec<HtmlToken>, i: usize) -> bool {
    return is_tag_start("rust", tokens, i);
}

fn next_is_rust_end(tokens: &Vec<HtmlToken>, i: usize) -> bool {
    return is_tag_end("rust", tokens, i);
}

fn next_is_markup_start(tokens: &Vec<HtmlToken>, i: usize) -> bool {
    return is_tag_start("markup", tokens, i);
}

fn peek(tokens: &Vec<HtmlToken>, i: usize, offset: usize) -> Option<HtmlToken> {
    let len = tokens.len();

    if i + offset >= len {
        return None;
    }

    Some(tokens[i + offset].clone())
}

fn is_tag_start(tag: &str, tokens: &Vec<HtmlToken>, i: usize) -> bool {
    if i + 2 >= tokens.len() {
        return false;
    }

    let start = &tokens[i].value;
    let name = &tokens[i + 1].value;
    let end = &tokens[i + 2].value;

    if start == "<" && name == tag && end == ">" {
        return true;
    }

    false
}

fn is_tag_end(tag: &str, tokens: &Vec<HtmlToken>, i: usize) -> bool {
    if i + 3 >= tokens.len() {
        return false;
    }

    let start = &tokens[i].value;
    let slash = &tokens[i + 1].value;
    let name = &tokens[i + 2].value;
    let end = &tokens[i + 3].value;

    if start == "<" && slash == "/" && name == tag && end == ">" {
        return true;
    }

    false
}

fn next_is_include_tag(tokens: &Vec<HtmlToken>, i: usize) -> bool {
    if i + 1 >= tokens.len() {
        return false;
    }

    let start = &tokens[i].value;
    let name = &tokens[i + 1].value;

    if start == "<" && name == "include" {
        return true;
    }

    false
}
