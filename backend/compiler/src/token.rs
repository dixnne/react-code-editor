use core::fmt;

// Se añade `Copy` para optimizar, ya que los enums son baratos de copiar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    CommentSingle,    // Comentario de una sola línea (ej. // comentario)
    CommentMultiLine, // Comentario de múltiples líneas (ej. /* comentario */)
    Keyword,          // Palabra reservada del lenguaje (ej. let, if, while)
    Identifier,       // Nombre dado por el usuario a variables, funciones, etc.
    Integer,          // Número entero (ej. 10, 42)
    Float,            // Número de punto flotante (ej. 3.14, 0.5)
    String,           // Cadena de caracteres (ej. "hola", 'mundo')
    Boolean,          // Valor booleano (true, false)
    
    // --- Operadores ---
    Plus,             // Operador de suma (+)
    Minus,            // Operador de resta (-)
    Asterisk,         // Operador de multiplicación (*)
    Slash,            // Operador de división (/)
    Equal,            // Operador de asignación (=)
    Greater,          // Operador mayor que (>)
    Less,             // Operador menor que (<)
    Exclamation,      // Signo de exclamación (!)
    Ampersand,        // Ampersand (&) - Generalmente para operaciones a nivel de bit
    Bar,              // Barra vertical (|) - Generalmente para operaciones a nivel de bit
    DoubleEqual,      // Operador de igualdad (==)
    GreaterEqual,     // Operador mayor o igual que (>=)
    LessEqual,        // Operador menor o igual que (<=)
    NotEqual,         // Operador de desigualdad (!= o <>)
    DoubleAmpersand,  // Operador lógico AND (&&)
    DoubleBar,        // Operador lógico OR (||)
    Increment,        // Operador de incremento (++)
    Decrement,        // Operador de decremento (--)
    
    // --- Operadores Especiales (Nombres Corregidos) ---
    Splat,            // @*
    Spread,           // ...+
    Pipe,             // |>
    Swap,             // <=>  <-- AÑADIDO

    // --- Delimitadores y Puntuación ---
    ArrowRight,       // Flecha (->)
    LeftParen,        // Paréntesis izquierdo (()
    RightParen,       // Paréntesis derecho ())
    LeftBrace,        // Llave izquierda ({)
    RightBrace,       // Llave derecha (})
    LeftBracket,      // Corchete izquierdo ([)
    RightBracket,     // Corchete derecho (])
    Comma,            // Coma (,)
    Semicolon,        // Punto y coma (;)
    Colon,            // Dos puntos (:)
    Dot,              // Punto (.)
    
    // --- Tokens Misceláneos ---
    Whitespace,       // Espacio en blanco, tabulación, etc.
    NewLine,          // Salto de línea
    EndOfFile,        // Marcador de fin de archivo/entrada
    Unknown,          // Token desconocido/inválido
}

impl TokenType {
    /// Convierte un string a un TokenType.
    /// Necesario para la comunicación con gRPC.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Keyword" => Some(TokenType::Keyword),
            "Identifier" => Some(TokenType::Identifier),
            "Integer" => Some(TokenType::Integer),
            "Float" => Some(TokenType::Float),
            "String" => Some(TokenType::String),
            "Boolean" => Some(TokenType::Boolean),
            "Plus" => Some(TokenType::Plus),
            "Minus" => Some(TokenType::Minus),
            "Asterisk" => Some(TokenType::Asterisk),
            "Slash" => Some(TokenType::Slash),
            "Equal" => Some(TokenType::Equal),
            "DoubleEqual" => Some(TokenType::DoubleEqual),
            "NotEqual" => Some(TokenType::NotEqual),
            "Less" => Some(TokenType::Less),
            "LessEqual" => Some(TokenType::LessEqual),
            "Greater" => Some(TokenType::Greater),
            "GreaterEqual" => Some(TokenType::GreaterEqual),
            "DoubleAmpersand" => Some(TokenType::DoubleAmpersand),
            "DoubleBar" => Some(TokenType::DoubleBar),
            "Exclamation" => Some(TokenType::Exclamation),
            "Splat" => Some(TokenType::Splat),
            "Spread" => Some(TokenType::Spread),
            "Pipe" => Some(TokenType::Pipe),
            "Swap" => Some(TokenType::Swap),
            "Increment" => Some(TokenType::Increment),
            "Decrement" => Some(TokenType::Decrement),
            "LeftParen" => Some(TokenType::LeftParen),
            "RightParen" => Some(TokenType::RightParen),
            "LeftBrace" => Some(TokenType::LeftBrace),
            "RightBrace" => Some(TokenType::RightBrace),
            "LeftBracket" => Some(TokenType::LeftBracket),
            "RightBracket" => Some(TokenType::RightBracket),
            "Comma" => Some(TokenType::Comma),
            "Semicolon" => Some(TokenType::Semicolon),
            "Colon" => Some(TokenType::Colon),
            "Dot" => Some(TokenType::Dot),
            "ArrowRight" => Some(TokenType::ArrowRight),
            _ => None,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexerToken {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl LexerToken {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}
