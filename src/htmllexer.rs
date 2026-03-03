// Copyright (c) 2026 Johan Mattsson
// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// See LICENSE-APACHE or LICENSE-MIT for details.

use std::fmt;
use std::iter::{Iterator, Peekable};

pub struct HtmlDynamicLexer<'a> {
    pub tokens: Vec<HtmlToken>,
    iterator: Peekable<std::str::Chars<'a>>,
    line_number: isize,
}

#[derive(Debug, Clone)]
pub struct HtmlToken {
    pub token_type: TokenType,
    pub value: String,
    pub white_space_before: String,
    pub white_space_after: String,
    pub line_number: isize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    StartBracket,
    EndBracket,
    StartParen,
    EndParen,
    StartTag,
    EndTag,
    OtherText,
    Dollar,
    Quote,
    Slash,
    Period,
    None,
}

impl fmt::Display for HtmlToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.white_space_before, self.value, self.white_space_after
        )
    }
}

impl<'a> HtmlDynamicLexer<'a> {
    pub fn new(input: &'a str, start_line: isize) -> HtmlDynamicLexer<'a> {
        HtmlDynamicLexer {
            tokens: Vec::new(),
            iterator: input.chars().peekable(),
            line_number: start_line,
        }
    }

    fn get_white_space(&mut self) -> String {
        let mut white_space = String::new();

        loop {
            match self.iterator.peek() {
                Some(&next) => {
                    if next == '\n' {
                        self.line_number += 1;
                    }

                    if next.is_whitespace() {
                        white_space.push(next);
                        self.iterator.next();
                    } else {
                        break;
                    }
                }
                _ => {
                    break;
                }
            }
        }

        white_space
    }

    pub fn lex(&mut self) {
        loop {
            let white_space_before = self.get_white_space();

            let next = self.peek();

            if next == '\0' {
                break;
            }

            let processed_token_type = match next {
                next if Self::is_other_text(next) => TokenType::OtherText,
                '$' => TokenType::Dollar,
                '"' => TokenType::Quote,
                '{' => TokenType::StartBracket,
                '}' => TokenType::EndBracket,
                '<' => TokenType::StartTag,
                '>' => TokenType::EndTag,
                '(' => TokenType::StartParen,
                ')' => TokenType::EndParen,
                '/' => TokenType::Slash,
                '.' => TokenType::Period,
                _ => panic!("Unhandled character: {}", next),
            };

            let current_text = match processed_token_type {
                TokenType::OtherText => self.process_other(),
                _ => {
                    let ch = next.to_string();
                    self.iterator.next();
                    ch
                }
            };

            let white_space_after = self.get_white_space();

            self.push(
                current_text,
                processed_token_type,
                white_space_before,
                white_space_after,
                self.line_number,
            );
        }
    }

    fn is_other_text(character: char) -> bool {
        match character {
            '"' => false,
            '{' => false,
            '}' => false,
            '<' => false,
            '>' => false,
            '$' => false,
            '(' => false,
            ')' => false,
            '/' => false,
            '.' => false,
            _ => true,
        }
    }

    #[cfg(test)]
    fn join(&self) -> String {
        let mut result = String::new();

        for token in &self.tokens {
            result.push_str(&token.white_space_before.as_str());
            result.push_str(&token.value.as_str());
            result.push_str(&token.white_space_after.as_str())
        }

        result
    }

    fn push(
        &mut self,
        text: String,
        token_type: TokenType,
        white_space_before: String,
        white_space_after: String,
        line_number: isize,
    ) {
        self.tokens.push(HtmlToken {
            token_type,
            value: text,
            white_space_before,
            white_space_after,
            line_number,
        });
    }

    fn process_other(&mut self) -> String {
        let mut current = String::new();

        if self.iterator.peek().unwrap().is_whitespace() {
            panic!("White space should be handled before other.");
        }

        loop {
            match self.iterator.peek() {
                Some(&next) if Self::is_other_text(next) => {
                    let breaker = next.is_whitespace();

                    if breaker {
                        break;
                    }

                    current.push(next);
                    self.iterator.next();
                }
                _ => {
                    break;
                }
            }
        }

        current
    }

    fn peek(&mut self) -> char {
        let character = self.iterator.peek();

        match character {
            Some(character) => *character,
            None => '\0',
        }
    }
}

#[test]
fn test_variable_substitution() {
    let text = "<h1>Hello $some_variable $(some_other_expression). $&some_ref. $(&some_struct.member). Tab tab:\t\t      and      \t\t</h1>";

    let mut lexer = HtmlDynamicLexer::new(text, 1);
    lexer.lex();

    let num_tokens = lexer.tokens.len();
    let joined = lexer.join();

    println!("{:?}", lexer.tokens);
    println!("Num tokens {}", num_tokens);
    println!("To lex: {:?}", &text);
    println!("joined: {:?}", joined);

    assert!(joined == text)
}
