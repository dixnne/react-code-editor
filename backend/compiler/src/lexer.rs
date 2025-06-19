use std::str::Chars; // Para iterar sobre caracteres de un string.
use std::iter::Peekable;
use crate::token::TokenType;
use std::collections::HashMap; // Para almacenar palabras clave y sus tipos de token.
use crate::token::LexerToken;

pub struct LexicalAnalyzer<'a> {
    input: Peekable<Chars<'a>>, // Iterador "peekable" sobre los caracteres del código fuente.
    line: usize,                // Número de línea actual.
    column: usize,              // Número de columna actual.
    keywords: HashMap<String, TokenType>, // Mapa de palabras clave del lenguaje.
}

// Implementación para LexicalAnalyzer.
impl<'a> LexicalAnalyzer<'a> {
    // Constructor para crear una nueva instancia de LexicalAnalyzer.
    pub fn new(source: &'a str) -> Self {
        let mut keywords = HashMap::new();
        // Inserta las palabras clave del lenguaje DreamC en el HashMap.
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
            input: source.chars().peekable(), // Convierte el string fuente en un iterador peekable.
            line: 1,                          // Inicia en la línea 1.
            column: 1,                        // Inicia en la columna 1.
            keywords,
        }
    }

    // Avanza al siguiente carácter en la entrada y lo devuelve, actualizando línea/columna.
    fn advance(&mut self) -> Option<char> {
        let current_char_opt = self.input.next();

        if let Some(ch) = current_char_opt {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        current_char_opt
    }

    // Devuelve una referencia al siguiente carácter sin consumirlo.
    fn peek(&mut self) -> Option<&char> {
        self.input.peek()
    }

    // Verifica si se ha llegado al final de la entrada.
    fn is_at_end(&mut self) -> bool {
        self.peek().is_none()
    }

    // Verifica si el siguiente carácter coincide con 'expected'. Si es así, lo consume.
    fn match_next(&mut self, expected: char) -> bool {
        if let Some(&next_char) = self.peek() {
            if next_char == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    // Escanea todo el código fuente y devuelve un vector de todos los tokens reconocidos.
    pub fn scan_tokens(&mut self) -> Vec<LexerToken> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            // Mientras no se llegue al final de la entrada.
            tokens.push(self.scan_token()); // Escanea y añade el siguiente token.
        }

        // Calcula la columna para el token EndOfFile.
        let eof_column = if tokens.last().map_or(false, |t| {
            t.lexeme == "\n" && t.token_type == TokenType::NewLine
        }) {
            1
        } else {
            if self.line == 1 && self.column == 1 && tokens.is_empty() {
                1
            } else {
                self.column
            }
        };
        // Añade un token EndOfFile al final de la lista.
        tokens.push(LexerToken::new(
            TokenType::EndOfFile,
            "".to_string(),
            self.line,
            eof_column,
        ));
        tokens
    }

    // Escanea y devuelve el siguiente token individual de la entrada.
    fn scan_token(&mut self) -> LexerToken {
        let start_line = self.line; // Guarda la línea de inicio del token.
        let start_column = self.column; // Guarda la columna de inicio del token.

        // Consume el carácter actual para análisis.
        let ch = match self.advance() {
            Some(c) => c,
            None => {
                panic!("scan_token called at end of file, this should be caught by is_at_end()")
            }
        };

        // Determina el tipo de token basado en el carácter actual (y posiblemente los siguientes).
        match ch {
            // Manejo de espacios en blanco.
            ' ' | '\t' | '\r' => {
                let mut lexeme = String::new();
                lexeme.push(ch);
                while let Some(&next_char) = self.peek() {
                    if next_char == ' ' || next_char == '\t' || next_char == '\r' {
                        lexeme.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                LexerToken::new(TokenType::Whitespace, lexeme, start_line, start_column)
            }
            // Manejo de saltos de línea.
            '\n' => LexerToken::new(
                TokenType::NewLine,
                "\n".to_string(),
                start_line,
                start_column,
            ),
            // Manejo de comentarios y operador de división.
            '/' => {
                if self.match_next('/') {
                    // Comentario de una línea.
                    let mut comment = String::from("//");
                    while let Some(&next_char) = self.peek() {
                        if next_char == '\n' {
                            break;
                        }
                        comment.push(self.advance().unwrap());
                    }
                    LexerToken::new(TokenType::CommentSingle, comment, start_line, start_column)
                } else if self.match_next('*') {
                    // Comentario multilínea.
                    let mut comment = String::from("/*");
                    let mut end_found = false;
                    while !self.is_at_end() {
                        if let Some(current_char) = self.advance() {
                            comment.push(current_char);
                            if current_char == '*' && self.peek() == Some(&'/') {
                                comment.push(self.advance().unwrap());
                                end_found = true;
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if end_found {
                        LexerToken::new(
                            TokenType::CommentMultiLine,
                            comment,
                            start_line,
                            start_column,
                        )
                    } else {
                        LexerToken::new(TokenType::Invalid, comment, start_line, start_column)
                    }
                } else {
                    // Operador de división.
                    LexerToken::new(TokenType::Slash, "/".to_string(), start_line, start_column)
                }
            }
            // Manejo de delimitadores simples.
            '(' => LexerToken::new(
                TokenType::LeftParen,
                "(".to_string(),
                start_line,
                start_column,
            ),
            ')' => LexerToken::new(
                TokenType::RightParen,
                ")".to_string(),
                start_line,
                start_column,
            ),
            '{' => LexerToken::new(
                TokenType::LeftBrace,
                "{".to_string(),
                start_line,
                start_column,
            ),
            '}' => LexerToken::new(
                TokenType::RightBrace,
                "}".to_string(),
                start_line,
                start_column,
            ),
            '[' => LexerToken::new(
                TokenType::LeftBracket,
                "[".to_string(),
                start_line,
                start_column,
            ),
            ']' => LexerToken::new(
                TokenType::RightBracket,
                "]".to_string(),
                start_line,
                start_column,
            ),
            ',' => LexerToken::new(TokenType::Comma, ",".to_string(), start_line, start_column),
            ';' => LexerToken::new(
                TokenType::Semicolon,
                ";".to_string(),
                start_line,
                start_column,
            ),
            ':' => LexerToken::new(TokenType::Colon, ":".to_string(), start_line, start_column),
            // Manejo del operador punto y el operador especial '...+'.
            '.' => {
                let mut lexeme = String::from(".");
                if self.match_next('.') {
                    lexeme.push('.');
                    if self.match_next('.') {
                        lexeme.push('.');
                        if self.match_next('+') {
                            lexeme.push('+');
                            LexerToken::new(TokenType::DotsPlus, lexeme, start_line, start_column)
                        } else {
                            LexerToken::new(TokenType::Invalid, lexeme, start_line, start_column)
                        }
                    } else {
                        LexerToken::new(TokenType::Dot, ".".to_string(), start_line, start_column)
                    }
                } else {
                    LexerToken::new(TokenType::Dot, ".".to_string(), start_line, start_column)
                }
            }
            // Manejo de operadores aritméticos y de incremento/decremento.
            '+' => {
                if self.match_next('+') {
                    LexerToken::new(
                        TokenType::Increment,
                        "++".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(TokenType::Plus, "+".to_string(), start_line, start_column)
                }
            }
            '-' => {
                if self.match_next('-') {
                    LexerToken::new(
                        TokenType::Decrement,
                        "--".to_string(),
                        start_line,
                        start_column,
                    )
                } else if self.match_next('>') {
                    LexerToken::new(
                        TokenType::ArrowRight,
                        "->".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(TokenType::Minus, "-".to_string(), start_line, start_column)
                }
            }
            '*' => LexerToken::new(
                TokenType::Asterisk,
                "*".to_string(),
                start_line,
                start_column,
            ),
            // Manejo de operadores de asignación y comparación.
            '=' => {
                if self.match_next('=') {
                    LexerToken::new(
                        TokenType::DoubleEqual,
                        "==".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(TokenType::Equal, "=".to_string(), start_line, start_column)
                }
            }
            '>' => {
                if self.match_next('=') {
                    LexerToken::new(
                        TokenType::GreaterEqual,
                        ">=".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(
                        TokenType::Greater,
                        ">".to_string(),
                        start_line,
                        start_column,
                    )
                }
            }
            '<' => {
                if self.match_next('=') {
                    LexerToken::new(
                        TokenType::LessEqual,
                        "<=".to_string(),
                        start_line,
                        start_column,
                    )
                } else if self.match_next('>') {
                    LexerToken::new(
                        TokenType::NotEqual,
                        "<>".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(TokenType::Less, "<".to_string(), start_line, start_column)
                }
            }
            '!' => {
                if self.match_next('=') {
                    LexerToken::new(
                        TokenType::NotEqual,
                        "!=".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(
                        TokenType::Exclamation,
                        "!".to_string(),
                        start_line,
                        start_column,
                    )
                }
            }
            // Manejo de operadores lógicos y especiales.
            '&' => {
                if self.match_next('&') {
                    LexerToken::new(
                        TokenType::DoubleAmpersand,
                        "&&".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(
                        TokenType::Ampersand,
                        "&".to_string(),
                        start_line,
                        start_column,
                    )
                }
            }
            '|' => {
                if self.match_next('|') {
                    LexerToken::new(
                        TokenType::DoubleBar,
                        "||".to_string(),
                        start_line,
                        start_column,
                    )
                } else if self.match_next('>') {
                    LexerToken::new(
                        TokenType::PipeGreater,
                        "|>".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(TokenType::Bar, "|".to_string(), start_line, start_column)
                }
            }
            '@' => {
                if self.match_next('*') {
                    LexerToken::new(
                        TokenType::AtAsterisk,
                        "@*".to_string(),
                        start_line,
                        start_column,
                    )
                } else {
                    LexerToken::new(
                        TokenType::Invalid,
                        "@".to_string(),
                        start_line,
                        start_column,
                    )
                }
            }
            // Manejo de cadenas de texto (strings).
            '\'' | '"' => {
                let quote_char = ch;
                let mut string_content = String::new();
                let mut lexeme_with_quotes = String::new();
                lexeme_with_quotes.push(quote_char);
                let mut terminated = false;

                while let Some(&nch) = self.peek() {
                    if nch == quote_char {
                        lexeme_with_quotes.push(self.advance().unwrap());
                        terminated = true;
                        break;
                    } else if nch == '\\' {
                        lexeme_with_quotes.push(self.advance().unwrap());
                        if let Some(&_escaped_char_peek) = self.peek() {
                            let escaped = self.advance().unwrap();
                            lexeme_with_quotes.push(escaped);
                            match escaped {
                                'n' => string_content.push('\n'),
                                't' => string_content.push('\t'),
                                'r' => string_content.push('\r'),
                                '\\' => string_content.push('\\'),
                                '\'' => string_content.push('\''),
                                '"' => string_content.push('"'),
                                _ => {
                                    string_content.push('\\');
                                    string_content.push(escaped);
                                }
                            }
                        } else {
                            return LexerToken::new(
                                TokenType::Invalid,
                                lexeme_with_quotes,
                                start_line,
                                start_column,
                            );
                        }
                    } else if nch == '\n' {
                        return LexerToken::new(
                            TokenType::Invalid,
                            lexeme_with_quotes,
                            start_line,
                            start_column,
                        );
                    } else {
                        let regular_char = self.advance().unwrap();
                        lexeme_with_quotes.push(regular_char);
                        string_content.push(regular_char);
                    }
                }

                if !terminated {
                    return LexerToken::new(
                        TokenType::Invalid,
                        lexeme_with_quotes,
                        start_line,
                        start_column,
                    );
                }
                LexerToken::new(TokenType::String, string_content, start_line, start_column)
            }
            // Manejo de identificadores y palabras clave.
            c if c.is_alphabetic() || c == '_' => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(&next_char) = self.peek() {
                    if next_char.is_alphanumeric() || next_char == '_' {
                        identifier.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                if let Some(keyword_type) = self.keywords.get(&identifier) {
                    LexerToken::new(keyword_type.clone(), identifier, start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Identifier, identifier, start_line, start_column)
                }
            }
            // Manejo de números (enteros y flotantes).
            c if c.is_digit(10) => {
                let mut number_str = String::new();
                number_str.push(c);
                while let Some(&next_char) = self.peek() {
                    if next_char.is_digit(10) {
                        number_str.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                if self.peek() == Some(&'.') {
                    let mut lookahead = self.input.clone();

                    if lookahead.next().is_some() {
                        if lookahead.peek() != Some(&'.') {
                            number_str.push(self.advance().unwrap());
                            let mut has_decimal_part = false;
                            while let Some(&next_char) = self.peek() {
                                if next_char.is_digit(10) {
                                    number_str.push(self.advance().unwrap());
                                    has_decimal_part = true;
                                } else {
                                    break;
                                }
                            }
                            if has_decimal_part {
                                return LexerToken::new(
                                    TokenType::Float,
                                    number_str,
                                    start_line,
                                    start_column,
                                );
                            } else {
                                return LexerToken::new(
                                    TokenType::Invalid,
                                    number_str,
                                    start_line,
                                    start_column,
                                );
                            }
                        }
                    }
                }
                LexerToken::new(TokenType::Integer, number_str, start_line, start_column)
            }
            // Manejo de cualquier otro carácter como inválido.
            _ => LexerToken::new(TokenType::Invalid, ch.to_string(), start_line, start_column),
        }
    }
}
