use std::str::Chars;
use std::iter::Peekable;
use crate::token::TokenType;
use std::collections::HashMap;
use crate::token::LexerToken;

pub struct LexicalAnalyzer<'a> {
    input: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenType>,
}

impl<'a> LexicalAnalyzer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("let".to_string(), TokenType::Keyword);
        keywords.insert("const".to_string(), TokenType::Keyword);
        keywords.insert("fn".to_string(), TokenType::Keyword);
        keywords.insert("if".to_string(), TokenType::Keyword);
        keywords.insert("else".to_string(), TokenType::Keyword);
        keywords.insert("while".to_string(), TokenType::Keyword);
        keywords.insert("struct".to_string(), TokenType::Keyword);
        keywords.insert("return".to_string(), TokenType::Keyword);
        keywords.insert("for".to_string(), TokenType::Keyword);
        keywords.insert("in".to_string(), TokenType::Keyword);
        keywords.insert("true".to_string(), TokenType::Boolean);
        keywords.insert("false".to_string(), TokenType::Boolean);

        Self {
            input: source.chars().peekable(),
            line: 1,
            column: 1,
            keywords,
        }
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.input.next() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.peek() == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<LexerToken> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            tokens.push(self.scan_token());
        }
        tokens.push(LexerToken::new(TokenType::EndOfFile, "".to_string(), self.line, self.column));
        tokens
    }

    fn scan_token(&mut self) -> LexerToken {
        let start_line = self.line;
        let start_column = self.column;

        let ch = match self.advance() {
            Some(c) => c,
            None => return LexerToken::new(TokenType::EndOfFile, "".to_string(), start_line, start_column),
        };

        match ch {
            ' ' | '\t' | '\r' => {
                let mut lexeme = String::from(ch);
                while let Some(' ' | '\t' | '\r') = self.peek() {
                    lexeme.push(self.advance().unwrap());
                }
                LexerToken::new(TokenType::Whitespace, lexeme, start_line, start_column)
            }
            '\n' => LexerToken::new(TokenType::NewLine, "\n".to_string(), start_line, start_column),
            '/' => {
                if self.match_next('/') {
                    let mut comment = String::new();
                    while let Some(c) = self.peek() {
                        if *c == '\n' { break; }
                        comment.push(self.advance().unwrap());
                    }
                    LexerToken::new(TokenType::CommentSingle, comment, start_line, start_column)
                } else if self.match_next('*') {
                    let mut comment = String::new();
                    while !self.is_at_end() {
                        if self.match_next('*') && self.match_next('/') {
                            return LexerToken::new(TokenType::CommentMultiLine, comment, start_line, start_column);
                        }
                        comment.push(self.advance().unwrap());
                    }
                    LexerToken::new(TokenType::Unknown, comment, start_line, start_column) // Unterminated comment
                } else {
                    LexerToken::new(TokenType::Slash, "/".to_string(), start_line, start_column)
                }
            }
            // --- Delimitadores y Operadores ---
            '(' => LexerToken::new(TokenType::LeftParen, "(".to_string(), start_line, start_column),
            ')' => LexerToken::new(TokenType::RightParen, ")".to_string(), start_line, start_column),
            '{' => LexerToken::new(TokenType::LeftBrace, "{".to_string(), start_line, start_column),
            '}' => LexerToken::new(TokenType::RightBrace, "}".to_string(), start_line, start_column),
            '[' => LexerToken::new(TokenType::LeftBracket, "[".to_string(), start_line, start_column),
            ']' => LexerToken::new(TokenType::RightBracket, "]".to_string(), start_line, start_column),
            ',' => LexerToken::new(TokenType::Comma, ",".to_string(), start_line, start_column),
            ';' => LexerToken::new(TokenType::Semicolon, ";".to_string(), start_line, start_column),
            ':' => LexerToken::new(TokenType::Colon, ":".to_string(), start_line, start_column),
            '.' => {
                if self.peek() == Some(&'.') {
                    self.advance();
                    if self.peek() == Some(&'.') {
                        self.advance();
                        if self.peek() == Some(&'+') {
                            self.advance();
                            return LexerToken::new(TokenType::Spread, "...+".to_string(), start_line, start_column);
                        }
                    }
                }
                LexerToken::new(TokenType::Dot, ".".to_string(), start_line, start_column)
            },
            '+' => if self.match_next('+') { LexerToken::new(TokenType::Increment, "++".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Plus, "+".to_string(), start_line, start_column) },
            '-' => if self.match_next('>') { LexerToken::new(TokenType::ArrowRight, "->".to_string(), start_line, start_column) } else if self.match_next('-') { LexerToken::new(TokenType::Decrement, "--".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Minus, "-".to_string(), start_line, start_column) },
            '*' => LexerToken::new(TokenType::Asterisk, "*".to_string(), start_line, start_column),
            '=' => if self.match_next('=') { LexerToken::new(TokenType::DoubleEqual, "==".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Equal, "=".to_string(), start_line, start_column) },
            '>' => if self.match_next('=') { LexerToken::new(TokenType::GreaterEqual, ">=".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Greater, ">".to_string(), start_line, start_column) },
            '<' => if self.match_next('=') {
                if self.match_next('>') { LexerToken::new(TokenType::Swap, "<=>".to_string(), start_line, start_column) }
                else { LexerToken::new(TokenType::LessEqual, "<=".to_string(), start_line, start_column) }
            } else if self.match_next('>') { LexerToken::new(TokenType::NotEqual, "<>".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Less, "<".to_string(), start_line, start_column) },
            '!' => if self.match_next('=') { LexerToken::new(TokenType::NotEqual, "!=".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Exclamation, "!".to_string(), start_line, start_column) },
            '&' => if self.match_next('&') { LexerToken::new(TokenType::DoubleAmpersand, "&&".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Ampersand, "&".to_string(), start_line, start_column) },
            '|' => if self.match_next('>') { LexerToken::new(TokenType::Pipe, "|>".to_string(), start_line, start_column) } else if self.match_next('|') { LexerToken::new(TokenType::DoubleBar, "||".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Bar, "|".to_string(), start_line, start_column) },
            '@' => if self.match_next('*') { LexerToken::new(TokenType::Splat, "@*".to_string(), start_line, start_column) } else { LexerToken::new(TokenType::Unknown, "@".to_string(), start_line, start_column) },
            // --- Literales ---
            '\'' | '"' => {
                let quote_char = ch;
                let mut content = String::new();
                while let Some(&next) = self.peek() {
                    if next == quote_char { break; }
                    if next == '\n' { return LexerToken::new(TokenType::Unknown, format!("{}{}", quote_char, content), start_line, start_column); } // Unterminated string
                    content.push(self.advance().unwrap());
                }
                self.advance(); // Consume closing quote
                LexerToken::new(TokenType::String, content, start_line, start_column)
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut identifier = String::from(c);
                while let Some(&next) = self.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        identifier.push(self.advance().unwrap());
                    } else { break; }
                }
                if let Some(token_type) = self.keywords.get(&identifier).cloned() {
                    LexerToken::new(token_type, identifier, start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Identifier, identifier, start_line, start_column)
                }
            }
            c if c.is_digit(10) => {
                let mut number_str = String::from(c);
                while let Some(&next) = self.peek() {
                    if next.is_digit(10) {
                        number_str.push(self.advance().unwrap());
                    } else { break; }
                }
                if self.peek() == Some(&'.') {
                     if self.input.clone().nth(1).map_or(false, |c| c.is_digit(10)) {
                        number_str.push(self.advance().unwrap()); // Consume '.'
                        while let Some(&next) = self.peek() {
                           if next.is_digit(10) {
                               number_str.push(self.advance().unwrap());
                           } else { break; }
                        }
                        return LexerToken::new(TokenType::Float, number_str, start_line, start_column);
                     }
                }
                LexerToken::new(TokenType::Integer, number_str, start_line, start_column)
            }
            _ => LexerToken::new(TokenType::Unknown, ch.to_string(), start_line, start_column),
        }
    }
}
