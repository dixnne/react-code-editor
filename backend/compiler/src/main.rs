// Importaciones de bibliotecas estándar y externas.
use std::collections::HashMap; // Para usar HashMaps (mapas de palabras clave).
use std::fmt; // Para formatear la salida (ej. para imprimir TokenType).
use std::str::Chars; // Para iterar sobre caracteres de un string.
use std::iter::Peekable; // Para poder "ver" el siguiente carácter sin consumirlo.

use tonic::{transport::Server, Request, Response, Status}; // Para el servidor y cliente gRPC.

// Módulo generado por tonic a partir del archivo .proto para la definición del servicio lexer.
pub mod lexer {
    tonic::include_proto!("lexer"); // Incluye el código Rust generado desde lexer.proto.
    
    // Constante que contiene el descriptor del archivo .proto, usado para reflexión del servicio gRPC.
    pub (crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lexer_descriptor");
}

// Importa los tipos necesarios del módulo lexer generado.
use lexer::{
    lexer_server::{Lexer, LexerServer}, // Traits y structs para el servidor gRPC.
    AnalyzeRequest, Token, TokenList, // Mensajes Protobuf para la comunicación.
};

// Enum que define todos los tipos de tokens que el analizador léxico puede reconocer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    CommentSingle,      // Comentario de una sola línea (ej. // comentario)
    CommentMultiLine,   // Comentario de múltiples líneas (ej. /* comentario */)
    Keyword,            // Palabra reservada del lenguaje (ej. let, if, while)
    Identifier,         // Nombre dado por el usuario a variables, funciones, etc.
    Integer,            // Número entero (ej. 10, 42)
    Float,              // Número de punto flotante (ej. 3.14, 0.5)
    String,             // Cadena de caracteres (ej. "hola", 'mundo')
    Plus,               // Operador de suma (+)
    Minus,              // Operador de resta (-)
    Asterisk,           // Operador de multiplicación (*)
    Slash,              // Operador de división (/)
    Equal,              // Operador de asignación o parte de igualdad (=)
    Greater,            // Operador mayor que (>)
    Less,               // Operador menor que (<)
    Exclamation,        // Signo de exclamación (!)
    Ampersand,          // Ampersand (&)
    Bar,                // Barra vertical (|)
    DoubleEqual,        // Operador de igualdad (==)
    GreaterEqual,       // Operador mayor o igual que (>=)
    LessEqual,          // Operador menor o igual que (<=)
    NotEqual,           // Operador de desigualdad (!= o <>)
    DoubleAmpersand,    // Operador lógico AND (&&)
    DoubleBar,          // Operador lógico OR (||)
    ArrowRight,         // Flecha (->)
    AtAsterisk,         // Arroba seguido de asterisco (@*)
    DotsPlus,           // Tres puntos seguidos de un más (...+)
    PipeGreater,        // Barra vertical seguida de mayor que (|>)
    Increment,          // Operador de incremento (++)
    Decrement,          // Operador de decremento (--)
    LeftParen,          // Paréntesis izquierdo (()
    RightParen,         // Paréntesis derecho ())
    LeftBrace,          // Llave izquierda ({)
    RightBrace,         // Llave derecha (})
    LeftBracket,        // Corchete izquierdo ([)
    RightBracket,       // Corchete derecho (])
    Comma,              // Coma (,)
    Semicolon,          // Punto y coma (;)
    Colon,              // Dos puntos (:)
    Dot,                // Punto (.)
    Whitespace,         // Espacio en blanco, tabulación, etc.
    NewLine,            // Salto de línea
    EndOfFile,          // Marcador de fin de archivo/entrada
    Invalid,            // Token no reconocido o inválido
}

// Implementación para poder imprimir TokenType usando `format!`.
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self) 
    }
}

// Estructura que representa un token individual reconocido por el lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct LexerToken {
    token_type: TokenType, // El tipo de token (del enum TokenType).
    lexeme: String,      // El texto original del token tal como aparece en el código fuente.
    line: usize,         // El número de línea donde se encontró el token.
    column: usize,       // El número de columna donde comienza el token.
}

// Implementación para LexerToken.
impl LexerToken {
    // Constructor para crear una nueva instancia de LexerToken.
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self { token_type, lexeme, line, column }
    }
}

// Estructura principal del analizador léxico.
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
        
        while !self.is_at_end() { // Mientras no se llegue al final de la entrada.
            tokens.push(self.scan_token()); // Escanea y añade el siguiente token.
        }
        
        // Calcula la columna para el token EndOfFile.
        let eof_column = if tokens.last().map_or(false, |t| t.lexeme == "\n" && t.token_type == TokenType::NewLine) {
            1
        } else {
            if self.line == 1 && self.column == 1 && tokens.is_empty() { 1 } else { self.column }
        };
        // Añade un token EndOfFile al final de la lista.
        tokens.push(LexerToken::new(TokenType::EndOfFile, "".to_string(), self.line, eof_column));
        tokens
    }
    
    // Escanea y devuelve el siguiente token individual de la entrada.
    fn scan_token(&mut self) -> LexerToken {
        let start_line = self.line;    // Guarda la línea de inicio del token.
        let start_column = self.column; // Guarda la columna de inicio del token.

        // Consume el carácter actual para análisis.
        let ch = match self.advance() {
            Some(c) => c,
            None => panic!("scan_token called at end of file, this should be caught by is_at_end()"),
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
            '\n' => {
                LexerToken::new(TokenType::NewLine, "\n".to_string(), start_line, start_column)
            }
            // Manejo de comentarios y operador de división.
            '/' => {
                if self.match_next('/') { // Comentario de una línea.
                    let mut comment = String::from("//");
                    while let Some(&next_char) = self.peek() {
                        if next_char == '\n' { break; }
                        comment.push(self.advance().unwrap());
                    }
                    LexerToken::new(TokenType::CommentSingle, comment, start_line, start_column)
                } else if self.match_next('*') { // Comentario multilínea.
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
                        } else { break; }
                    }
                    if end_found {
                        LexerToken::new(TokenType::CommentMultiLine, comment, start_line, start_column)
                    } else {
                        LexerToken::new(TokenType::Invalid, comment, start_line, start_column) 
                    }
                } else { // Operador de división.
                    LexerToken::new(TokenType::Slash, "/".to_string(), start_line, start_column)
                }
            }
            // Manejo de delimitadores simples.
            '(' => LexerToken::new(TokenType::LeftParen, "(".to_string(), start_line, start_column),
            ')' => LexerToken::new(TokenType::RightParen, ")".to_string(), start_line, start_column),
            '{' => LexerToken::new(TokenType::LeftBrace, "{".to_string(), start_line, start_column),
            '}' => LexerToken::new(TokenType::RightBrace, "}".to_string(), start_line, start_column),
            '[' => LexerToken::new(TokenType::LeftBracket, "[".to_string(), start_line, start_column),
            ']' => LexerToken::new(TokenType::RightBracket, "]".to_string(), start_line, start_column),
            ',' => LexerToken::new(TokenType::Comma, ",".to_string(), start_line, start_column),
            ';' => LexerToken::new(TokenType::Semicolon, ";".to_string(), start_line, start_column),
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
                    LexerToken::new(TokenType::Increment, "++".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Plus, "+".to_string(), start_line, start_column)
                }
            },
            '-' => {
                if self.match_next('-') {
                    LexerToken::new(TokenType::Decrement, "--".to_string(), start_line, start_column)
                } else if self.match_next('>') {
                    LexerToken::new(TokenType::ArrowRight, "->".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Minus, "-".to_string(), start_line, start_column)
                }
            },
            '*' => LexerToken::new(TokenType::Asterisk, "*".to_string(), start_line, start_column),
            // Manejo de operadores de asignación y comparación.
            '=' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::DoubleEqual, "==".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Equal, "=".to_string(), start_line, start_column)
                }
            }
            '>' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::GreaterEqual, ">=".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Greater, ">".to_string(), start_line, start_column)
                }
            }
            '<' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::LessEqual, "<=".to_string(), start_line, start_column)
                } else if self.match_next('>') { 
                    LexerToken::new(TokenType::NotEqual, "<>".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Less, "<".to_string(), start_line, start_column)
                }
            }
            '!' => {
                if self.match_next('=') {
                    LexerToken::new(TokenType::NotEqual, "!=".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Exclamation, "!".to_string(), start_line, start_column)
                }
            }
            // Manejo de operadores lógicos y especiales.
            '&' => {
                if self.match_next('&') {
                    LexerToken::new(TokenType::DoubleAmpersand, "&&".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Ampersand, "&".to_string(), start_line, start_column)
                }
            }
            '|' => {
                if self.match_next('|') {
                    LexerToken::new(TokenType::DoubleBar, "||".to_string(), start_line, start_column)
                } else if self.match_next('>') {
                    LexerToken::new(TokenType::PipeGreater, "|>".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Bar, "|".to_string(), start_line, start_column)
                }
            }
            '@' => {
                if self.match_next('*') {
                    LexerToken::new(TokenType::AtAsterisk, "@*".to_string(), start_line, start_column)
                } else {
                    LexerToken::new(TokenType::Invalid, "@".to_string(), start_line, start_column)
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
                            return LexerToken::new(TokenType::Invalid, lexeme_with_quotes, start_line, start_column);
                        }
                    } else if nch == '\n' { 
                        return LexerToken::new(TokenType::Invalid, lexeme_with_quotes, start_line, start_column);
                    } else {
                        let regular_char = self.advance().unwrap();
                        lexeme_with_quotes.push(regular_char);
                        string_content.push(regular_char);
                    }
                }
            
                if !terminated {
                    return LexerToken::new(TokenType::Invalid, lexeme_with_quotes, start_line, start_column);
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
                                return LexerToken::new(TokenType::Float, number_str, start_line, start_column);
                            } else {
                                return LexerToken::new(TokenType::Invalid, number_str, start_line, start_column);
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

// Estructura que implementa el servicio gRPC Lexer.
#[derive(Debug, Default)]
pub struct LexerService {}

// Implementación del trait Lexer para LexerService.
#[tonic::async_trait]
impl Lexer for LexerService {
    // Método asíncrono que maneja las solicitudes de análisis léxico.
    async fn analyze(&self, request: Request<AnalyzeRequest>) -> std::result::Result<Response<TokenList>, Status> {
        println!("Got request: {:?}", request); // Imprime la solicitud recibida (para depuración).
        
        let input_str = request.into_inner().input; // Extrae el string de entrada de la solicitud.
        let mut analyzer = LexicalAnalyzer::new(&input_str); // Crea una nueva instancia del analizador.
        let tokens = analyzer.scan_tokens(); // Escanea los tokens.
        
        // Convierte los LexerToken de Rust al formato Token de Protobuf, filtrando espacios y saltos de línea.
        let token_list_proto = tokens.into_iter()
            .filter(|t| t.token_type != TokenType::Whitespace && t.token_type != TokenType::NewLine)
            .map(|t| Token {
                token_type: t.token_type.to_string(),
                lexeme: t.lexeme,
                line: t.line as u32,
                column: t.column as u32,
            })
            .collect::<Vec<_>>();
        
        // Devuelve la lista de tokens en una respuesta gRPC.
        Ok(Response::new(TokenList { tokens: token_list_proto }))
    }
}

// Función principal asíncrona que inicia el servidor gRPC.
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?; // Define la dirección y puerto del servidor.
    let lexer_service = LexerService::default(); // Crea una instancia del servicio.

    // Configura el servicio de reflexión gRPC para permitir la introspección del servicio.
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(lexer::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    // Construye e inicia el servidor gRPC.
    Server::builder()
        .add_service(LexerServer::new(lexer_service)) // Añade el servicio Lexer.
        .add_service(reflection_service) // Añade el servicio de reflexión.
        .serve(addr) // Inicia el servidor en la dirección especificada.
        .await?;

    println!("Server listening on {}", addr); // Mensaje de confirmación.

    Ok(())
}

// Módulo de pruebas unitarias.
#[cfg(test)]
mod tests {
    use super::{LexicalAnalyzer, TokenType, LexerToken}; // Importa componentes para las pruebas.

    // Función auxiliar para crear instancias de LexerToken de forma concisa en las pruebas.
    fn tk(token_type: TokenType, lexeme: &str, line: usize, column: usize) -> LexerToken {
        LexerToken::new(token_type, lexeme.to_string(), line, column)
    }

    // Prueba para una declaración simple de variable.
    #[test]
    fn test_simple_declaration() {
        let input = "let x = 10;";
        let mut analyzer = LexicalAnalyzer::new(input);
        let tokens = analyzer.scan_tokens();
        
        let filtered_tokens: Vec<LexerToken> = tokens.into_iter()
            .filter(|t| t.token_type != TokenType::Whitespace && t.token_type != TokenType::NewLine)
            .collect();

         let filtered_expected = vec![
            tk(TokenType::Keyword, "let", 1, 1),
            tk(TokenType::Identifier, "x", 1, 5), 
            tk(TokenType::Equal, "=", 1, 7),     
            tk(TokenType::Integer, "10", 1, 9),   
            tk(TokenType::Semicolon, ";", 1, 11),
            tk(TokenType::EndOfFile, "", 1, 12),
        ];
        assert_eq!(filtered_tokens, filtered_expected, "Fallo en test_simple_declaration");
    }

    // Prueba para varios operadores.
    #[test]
    fn test_operators() {
        let input = "+ - * / == != < > <= >= && || ! ++ -- += -> @* ...+ |>"; 
        let mut analyzer = LexicalAnalyzer::new(input);
        let tokens = analyzer.scan_tokens();

         let filtered_tokens: Vec<LexerToken> = tokens.into_iter()
            .filter(|t| t.token_type != TokenType::Whitespace && t.token_type != TokenType::NewLine)
            .collect();

         let filtered_expected = vec![
            tk(TokenType::Plus, "+", 1, 1),
            tk(TokenType::Minus, "-", 1, 3),
            tk(TokenType::Asterisk, "*", 1, 5),
            tk(TokenType::Slash, "/", 1, 7),
            tk(TokenType::DoubleEqual, "==", 1, 9),
            tk(TokenType::NotEqual, "!=", 1, 12),
            tk(TokenType::Less, "<", 1, 15),
            tk(TokenType::Greater, ">", 1, 17),
            tk(TokenType::LessEqual, "<=", 1, 19),
            tk(TokenType::GreaterEqual, ">=", 1, 22),
            tk(TokenType::DoubleAmpersand, "&&", 1, 25),
            tk(TokenType::DoubleBar, "||", 1, 28),
            tk(TokenType::Exclamation, "!", 1, 31),
            tk(TokenType::Increment, "++", 1, 33),
            tk(TokenType::Decrement, "--", 1, 36),
            tk(TokenType::Plus, "+", 1, 39), 
            tk(TokenType::Equal, "=", 1, 40),
            tk(TokenType::ArrowRight, "->", 1, 42),
            tk(TokenType::AtAsterisk, "@*", 1, 45),
            tk(TokenType::DotsPlus, "...+", 1, 48),
            tk(TokenType::PipeGreater, "|>", 1, 53),
            tk(TokenType::EndOfFile, "", 1, 55),
        ];
        assert_eq!(filtered_tokens, filtered_expected, "Fallo en test_operators");
    }

    // Prueba para comentarios de una y múltiples líneas.
    #[test]
    fn test_comments() {
        let input = "// Line comment\n/* Block \n comment */ identifier";
        let mut analyzer = LexicalAnalyzer::new(input);
        let tokens = analyzer.scan_tokens();

         let expected_tokens = vec![
            tk(TokenType::CommentSingle, "// Line comment", 1, 1),
            tk(TokenType::NewLine, "\n", 1, 16),
            tk(TokenType::CommentMultiLine, "/* Block \n comment */", 2, 1),
            tk(TokenType::Whitespace, " ", 3, 12),
            tk(TokenType::Identifier, "identifier", 3, 13),
            tk(TokenType::EndOfFile, "", 3, 23),
        ];
        assert_eq!(tokens, expected_tokens, "Fallo en test_comments");
    }

    // Prueba para un fragmento de código más completo.
    #[test]
    fn test_comprehensive_code() {
        let input = r#"
        // Ejemplo completo
        let pi: float = 3.14;
        fn main() -> int {
            /* Multi
               line */
            if pi > 3 && true {
                let msg = "OK";
                x++;
            }
            return 0;
        }
        "#;
        let mut analyzer = LexicalAnalyzer::new(input);
        let tokens = analyzer.scan_tokens();

        let filtered_tokens: Vec<LexerToken> = tokens.into_iter()
            .filter(|t| t.token_type != TokenType::Whitespace && t.token_type != TokenType::NewLine && t.token_type != TokenType::CommentSingle && t.token_type != TokenType::CommentMultiLine)
            .collect();

        let expected_tokens = vec![
            tk(TokenType::Keyword, "let", 3, 9),
            tk(TokenType::Identifier, "pi", 3, 13),
            tk(TokenType::Colon, ":", 3, 15),
            tk(TokenType::Identifier, "float", 3, 17),
            tk(TokenType::Equal, "=", 3, 23),
            tk(TokenType::Float, "3.14", 3, 25),
            tk(TokenType::Semicolon, ";", 3, 29),
            tk(TokenType::Keyword, "fn", 4, 9),
            tk(TokenType::Identifier, "main", 4, 12),
            tk(TokenType::LeftParen, "(", 4, 16),
            tk(TokenType::RightParen, ")", 4, 17),
            tk(TokenType::ArrowRight, "->", 4, 19),
            tk(TokenType::Identifier, "int", 4, 22),
            tk(TokenType::LeftBrace, "{", 4, 26),
            tk(TokenType::Keyword, "if", 7, 13),
            tk(TokenType::Identifier, "pi", 7, 16),
            tk(TokenType::Greater, ">", 7, 19),
            tk(TokenType::Integer, "3", 7, 21),
            tk(TokenType::DoubleAmpersand, "&&", 7, 23),
            tk(TokenType::Identifier, "true", 7, 26),
            tk(TokenType::LeftBrace, "{", 7, 31),
            tk(TokenType::Keyword, "let", 8, 17),
            tk(TokenType::Identifier, "msg", 8, 21),
            tk(TokenType::Equal, "=", 8, 25),
            tk(TokenType::String, "OK", 8, 27),
            tk(TokenType::Semicolon, ";", 8, 31),
            tk(TokenType::Identifier, "x", 9, 17),
            tk(TokenType::Increment, "++", 9, 18),
            tk(TokenType::Semicolon, ";", 9, 20),
            tk(TokenType::RightBrace, "}", 10, 13),
            tk(TokenType::Keyword, "return", 11, 13),
            tk(TokenType::Integer, "0", 11, 20),
            tk(TokenType::Semicolon, ";", 11, 21),
            tk(TokenType::RightBrace, "}", 12, 9),
            tk(TokenType::EndOfFile, "", 13, 9),
        ];
        assert_eq!(filtered_tokens, expected_tokens, "Fallo en test_comprehensive_code");
    }
}
