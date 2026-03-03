// Copyright (c) 2026 Johan Mattsson
// License: MIT

use quote::quote;
use std::usize;

use crate::htmlcontentbuilder::{ErrType, HtmlError};
use crate::htmllexer::{HtmlDynamicLexer, HtmlToken, TokenType};

pub fn html_quotes(html_content: &str, line_number: isize) -> Result<String, HtmlError> {
    let mut updater = HtmlDynamicLexer::new(html_content, line_number);
    updater.lex();

    let mut code_generator = HtmlQuotes::new(&updater);
    code_generator.generate_rust_code()?;

    return Ok(code_generator.result);
}

struct HtmlQuotes<'a> {
    lexer: &'a HtmlDynamicLexer<'a>,
    result: String,
    current_index: usize,
}

impl<'a> HtmlQuotes<'a> {
    fn new(lexer: &'a HtmlDynamicLexer<'a>) -> HtmlQuotes<'a> {
        HtmlQuotes {
            lexer,
            result: String::new(),
            current_index: 0,
        }
    }

    fn get(&self, i: usize) -> Option<&HtmlToken> {
        if i >= self.lexer.tokens.len() {
            return None;
        }

        return Some(&self.lexer.tokens[i]);
    }

    fn generate_rust_code(&mut self) -> Result<(), HtmlError> {
        let mut pass_through = String::new();

        loop {
            let current = self.get(self.current_index);

            if current.is_none() {
                self.add_pass_through(&mut pass_through);
                return Ok(());
            }

            let current_value = current.unwrap();

            if self.next_is_variable_expression() {
                self.add_pass_through(&mut pass_through);
                let variable_expression_tokens = self.process_variables_expression()?;
                self.result.push_str(variable_expression_tokens.as_str());
                // advance past  $ ( identifier ) is done in process_variables_expression
            } else if self.next_is_variable() {
                self.add_pass_through(&mut pass_through);
                let variable_tokens = self.process_variables()?;
                self.result.push_str(variable_tokens.as_str());
                self.current_index += 2; // advance past $ identifier
            } else {
                pass_through.push_str(current_value.to_string().as_str());
                self.current_index += 1;
            }
        }
    }

    fn get_type(&self, i: usize) -> TokenType {
        let token = self.get(i);

        match token {
            Some(t) => t.token_type.clone(),
            None => TokenType::None,
        }
    }

    fn add_pass_through(&mut self, pass_through: &mut String) {
        if pass_through == "" {
            return;
        }

        let html = quote! {
            __html_macro_content.push_str(#pass_through);
        };

        self.result.push_str(html.to_string().as_str());
        self.result.push_str("\n");
        pass_through.clear();
    }

    fn next_is_variable(&self) -> bool {
        let type_current = self.get_type(self.current_index);
        let type_ahead1 = self.get_type(self.current_index + 1);

        if type_current == TokenType::Dollar && type_ahead1 == TokenType::OtherText {
            return true;
        }

        false
    }

    fn next_is_variable_expression(&self) -> bool {
        let type_current = self.get_type(self.current_index);
        let type_ahead1 = self.get_type(self.current_index + 1);
        let type_ahead2 = self.get_type(self.current_index + 2);

        if type_current == TokenType::Dollar
            && type_ahead1 == TokenType::StartParen
            && type_ahead2 == TokenType::OtherText
        {
            return true;
        }

        false
    }

    fn process_variables_expression(&mut self) -> Result<String, HtmlError> {
        let mut idx = self.current_index;

        let ahead1 = self.get(idx + 1);
        let ahead2 = self.get(idx + 2);

        if ahead1.is_none() {
            return Err(HtmlError {
                message: "Expecting a variable or expression name inside $() in html macro".into(),
                hint: String::new(),
                line: -1,
                err_type: ErrType::VarExpr,
            });
        }

        if ahead2.is_none() {
            return Err(HtmlError {
                message: "Expecting $()".into(),
                hint: String::new(),
                line: -1,
                err_type: ErrType::VarExpr,
            });
        }

        // skip '$' and '('
        idx += 2;

        let mut paren_count = 0;
        let mut expr_string = String::new();

        loop {
            let token_opt = self.get(idx);

            if token_opt.is_none() {
                return Err(HtmlError {
                    message: "Expecting full $()".into(),
                    hint: String::new(),
                    line: -1,
                    err_type: ErrType::VarExpr,
                });
            }

            let token = token_opt.unwrap();

            if token.value == ")" {
                if paren_count == 0 {
                    break;
                } else {
                    paren_count -= 1;
                }
            } else if token.value == "(" {
                paren_count += 1;
            }

            expr_string.push_str(token.to_string().as_str());
            idx += 1;
        }

        // idx is at the closing ')'; set current_index to the token after it
        self.current_index = idx + 1;
        let result = Self::get_escaped_code(expr_string);

        Ok(result)
    }

    fn process_variables(&self) -> Result<String, HtmlError> {
        let ahead1 = self.get(self.current_index + 1);

        if ahead1.is_none() {
            return Err(HtmlError {
                message: "Expecting a variable after $ in html macro.".into(),
                hint: String::new(),
                line: -1,
                err_type: ErrType::VarSingleExpr,
            });
        }

        let symbol = ahead1.unwrap();
        let identifier = &symbol.value;

        let mut ident = identifier.clone();
        while ident.ends_with(|c: char| !c.is_alphanumeric() && c != '_') {
            ident.pop();
        }

        if ident.is_empty() {
            let err_mess = format!("Not a valid variable name: {:?}.", identifier);

            return Err(HtmlError {
                message: err_mess,
                hint: String::new(),
                line: -1,
                err_type: ErrType::VarSingleExpr,
            });
        }

        let result = Self::get_escaped_code(ident);
        Ok(result)
    }

    fn get_escaped_code(code: String) -> String {
        let mut result = String::new();

        result.push_str("let __html_variable = format!(\"{}\", ");
        result.push_str(code.as_str());
        result.push_str(");\n");

        result.push_str(
            r#"
            if __html_variable.chars().any(|c| ['<', '>', '/', '&', '\'', '"', '\\', '='].contains(&c)) {
                let mut __escaped = String::with_capacity(2 * __html_variable.len());

                for c in __html_variable.as_str().chars() {
                    match c {
                        '<' => __escaped.push_str("&lt;"),
                        '>' => __escaped.push_str("&gt;"),
                        '/' => __escaped.push_str("&#x2f;"),
                        '&' => __escaped.push_str("&amp;"),
                        '\'' => __escaped.push_str("&apos;"),
                        '"' => __escaped.push_str("&quot;"),
                        '\\' => __escaped.push_str("&#x5c;"),
                        '=' => __escaped.push_str("&equals;"),
                        _ => __escaped.push(c),
                    }
                }

                __html_macro_content.push_str(&__escaped.as_str());
            } else {
                __html_macro_content.push_str(&__html_variable.as_str());
            }
        "#,
        );

        result
    }
}
