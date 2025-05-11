use std::collections::HashMap;
use std::fmt;
use std::str::Chars;
use std::sync::Arc;
use std::io::Result;

use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

// Proto-generated code
pub mod lexer {
    tonic::include_proto!("lexer");
    
    pub (crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lexer_descriptor");
}

use lexer::{
    lexer_server::{Lexer, LexerServer},
    AnalyzeRequest, Token, TokenList,
};


#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Comments
    CommentSingle,
    CommentMultiLine,
    
    // Keywords
    Keyword,
    
    // Identifiers and literals
    Identifier,
    Integer,
    Float,
    String,
    
    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Equal,
    Greater,
    Less,
    Exclamation,
    Ampersand,
    Bar,
    DoubleEqual,
    GreaterEqual,
    LessEqual,
    NotEqual,
    DoubleAmpersand,
    DoubleBar,
    ArrowRight,
    AtAsterisk,
    DotsPlus,
    PipeGreater,
    Increment,
    Decrement,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    
    // Special
    Whitespace,
    NewLine,
    EndOfFile,
    Invalid,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self) 
    }
}

#[derive(Debug, Clone)]
pub struct LexerToken {
    token_type: TokenType,
    lexeme: String,
    line: usize,
    column: usize,
}

impl LexerToken {
    fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self { token_type, lexeme, line, column }
    }
}

pub struct LexicalAnalyzer<'a> {
    input: Chars<'a>,
    current: Option<char>,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenType>,
}

impl<'a> LexicalAnalyzer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let current = chars.next();
        
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
        
        Self {
            input: chars,
            current,
            line: 1,
            column: 1,
            keywords,
        }
    }
    
    fn advance(&mut self) -> Option<char> {
        let current = self.current;
        self.current = self.input.next();
        if let Some(ch) = current {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        current
    }
    
    fn peek(&self) -> Option<char> {
        self.current
    }
    
    fn is_at_end(&self) -> bool {
        self.current.is_none()
    }
    
    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current != Some(expected) {
            false
        } else {
            self.advance();
            true
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
        let ch = self.advance().unwrap();
        let start_column = self.column - 1;
        
        match ch {
            // Whitespace
            ' ' | '\t' | '\r' => {
                LexerToken::new(TokenType::Whitespace, ch.to_string(), self.line, start_column)
            }
            
            '\n' => {
                LexerToken::new(TokenType::NewLine, "\n".to_string(), self.line - 1, start_column)
            }
            
            // Comments
            '/' => {
                if self.match_next('/') {
                    // Single-line comment
                    let mut comment = "//".to_string();
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        comment.push(self.advance().unwrap());
                    }
                    LexerToken::new(TokenType::CommentSingle, comment, self.line, start_column)
                } else if self.match_next('*') {
                    // Multi-line comment
                    let mut comment = "/*".to_string();
                    let mut end_found = false;
                    
                    while !end_found && !self.is_at_end() {
                        let current_char = self.advance().unwrap();
                        comment.push(current_char);
                        
                        if current_char == '*' && self.peek() == Some('/') {
                            comment.push(self.advance().unwrap());
                            end_found = true;
                        }
                    }
                    
                    if end_found {
                        LexerToken::new(TokenType::CommentMultiLine, comment, self.line, start_column)
                    } else {
                        LexerToken::new(TokenType::Invalid, comment, self.line, start_column)
                    }
                } else {
                    LexerToken::new(TokenType::Slash, "/".to_string(), self.line, start_column)
                }
            }
            
            // Delimiters
            '(' => LexerToken::new(TokenType::LeftParen, "(".to_string(), self.line, start_column),
            ')' => LexerToken::new(TokenType::RightParen, ")".to_string(), self.line, start_column),
            '{' => LexerToken::new(TokenType::LeftBrace, "{".to_string(), self.line, start_column),
            '}' => LexerToken::new(TokenType::RightBrace, "}".to_string(), self.line, start_column),
            '[' => LexerToken::new(TokenType::LeftBracket, "[".to_string(), self.line, start_column),
            ']' => LexerToken::new(TokenType::RightBracket, "]".to_string(), self.line, start_column),
            ',' => LexerToken::new(TokenType::Comma, ",".to_string(), self.line, start_column),
            ';' => LexerToken::new(TokenType::Semicolon, ";".to_string(), self.line, start_column),
            ':' => LexerToken::new(TokenType::Colon, ":".to_string(), self.line, start_column),
            '.' => {
                if self.match_next('.') && self.match_next('.') {
                    if self.match_next('+') {
                        LexerToken::new(TokenType::DotsPlus, "...+".to_string(), self.line, start_column)
                    } else {
                        // Just consume the three dots
                        LexerToken::new(TokenType::Invalid, "...".to_string(), self.line, start_column)
                    }
                } else {
                    LexerToken::new(TokenType::Dot, ".".to_string(), self.line, start_column)
                }
            }
            
            // Operators
            '+' => {
                if self.match_next('+') {
                    LexerToken::new(TokenType::Increment, "++".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Plus, "+".to_string(), self.line, start_column)
                }
            },
            '-' => {
                if self.match_next('-') {
                    LexerToken::new(TokenType::Decrement, "--".to_string(), self.line, start_column)
                } else if self.match_next('>') {
                    LexerToken::new(TokenType::ArrowRight, "->".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Minus, "-".to_string(), self.line, start_column)
                }
            },
            '*' => LexerToken::new(TokenType::Asterisk, "*".to_string(), self.line, start_column),
            '=' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::DoubleEqual, "==".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Equal, "=".to_string(), self.line, start_column)
                }
            }
            '>' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::GreaterEqual, ">=".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Greater, ">".to_string(), self.line, start_column)
                }
            }
            '<' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::LessEqual, "<=".to_string(), self.line, start_column)
                } else if self.match_next('>') {
                    LexerToken::new(TokenType::NotEqual, "<>".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Less, "<".to_string(), self.line, start_column)
                }
            }
            '!' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::NotEqual, "!=".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Exclamation, "!".to_string(), self.line, start_column)
                }
            }
            '&' => {
                if self.match_next('&') {
                    LexerToken::new(TokenType::DoubleAmpersand, "&&".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Ampersand, "&".to_string(), self.line, start_column)
                }
            }
            '|' => {
                if self.match_next('|') {
                    LexerToken::new(TokenType::DoubleBar, "||".to_string(), self.line, start_column)
                } else if self.match_next('>') {
                    LexerToken::new(TokenType::PipeGreater, "|>".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Bar, "|".to_string(), self.line, start_column)
                }
            }
            '@' => {
                if self.match_next('*') {
                    LexerToken::new(TokenType::AtAsterisk, "@*".to_string(), self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Invalid, "@".to_string(), self.line, start_column)
                }
            }
            
            // Strings
            '\'' | '"' => {
                let quote_char = ch;
                let mut string = String::new();
                let mut terminated = false;
            
                while let Some(nch) = self.peek() {
                    if nch == quote_char {
                        self.advance();
                        terminated = true;
                        break;
                    } else if nch == '\\' {
                        // Handle escape sequences
                        self.advance(); // Consume backslash
                        if let Some(escaped) = self.advance() {
                            match escaped {
                                '\\' => string.push('\\'),
                                'n' => string.push('\n'),
                                't' => string.push('\t'),
                                'r' => string.push('\r'),
                                '\'' => string.push('\''),
                                '"' => string.push('"'),
                                other => string.push(other),
                            }
                        } else {
                            return LexerToken::new(TokenType::Invalid, "Unterminated escape sequence".to_string(), self.line, start_column);
                        }
                    } else if nch == '\n' {
                        // Treat newline as an error in strings
                        return LexerToken::new(TokenType::Invalid, "Newline in string literal".to_string(), self.line, start_column);
                    } else {
                        string.push(self.advance().unwrap());
                    }
                }
            
                if !terminated {
                    return LexerToken::new(TokenType::Invalid, "Unterminated string literal".to_string(), self.line, start_column);
                }
            
                LexerToken::new(TokenType::String, string, self.line, start_column)
            }

            // Identifiers and keywords
            ch if ch.is_alphabetic() || ch == '_' => {
                let mut identifier = String::new();
                identifier.push(ch);
                
                while let Some(next_char) = self.peek() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        identifier.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                
                if self.keywords.contains_key(&identifier) {
                    LexerToken::new(TokenType::Keyword, identifier, self.line, start_column)
                } else {
                    LexerToken::new(TokenType::Identifier, identifier, self.line, start_column)
                }
            }
            
            // Numbers (simplified to just integers and floats)
            ch if ch.is_digit(10) => {
                let mut number = String::new();
                number.push(ch);
                
                // Integer part
                while let Some(next_char) = self.peek() {
                    if next_char.is_digit(10) {
                        number.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                
                // Check for float
                if self.peek() == Some('.') {
                    number.push(self.advance().unwrap());
                    
                    // Decimal part
                    let mut has_decimal_digit = false;
                    while let Some(next_char) = self.peek() {
                        if next_char.is_digit(10) {
                            number.push(self.advance().unwrap());
                            has_decimal_digit = true;
                        } else {
                            break;
                        }
                    }
                    
                    if has_decimal_digit {
                        return LexerToken::new(TokenType::Float, number, self.line, start_column);
                    } else {
                        return LexerToken::new(TokenType::Invalid, number, self.line, start_column);
                    }
                }
                
                LexerToken::new(TokenType::Integer, number, self.line, start_column)
            }
            
            // Invalid characters
            _ => LexerToken::new(TokenType::Invalid, ch.to_string(), self.line, start_column),
        }
    }
}

// gRPC service implementation
#[derive(Debug, Default)]
pub struct LexerService {}

#[tonic::async_trait]
impl Lexer for LexerService {
    async fn analyze(&self, request: Request<AnalyzeRequest>) -> std::result::Result<Response<TokenList>, Status> {
        println!("Got request: {:?}", request);
        
        let input = request.into_inner().input;
        let mut analyzer = LexicalAnalyzer::new(&input);
        let tokens = analyzer.scan_tokens();
        
        let token_list = tokens.into_iter()
            .filter(|t| t.token_type != TokenType::Whitespace)
            .map(|t| Token {
                token_type: t.token_type.to_string(),
                lexeme: t.lexeme,
                line: t.line as u32,
                column: t.column as u32,
            })
            .collect::<Vec<_>>();
        
        Ok(Response::new(TokenList { tokens: token_list }))
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let lexer_service = LexerService::default();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(lexer::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    Server::builder()
        .add_service(LexerServer::new(lexer_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    println!("Server listening on {}", addr);

    Ok(())
}