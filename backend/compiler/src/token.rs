use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    CommentSingle,    // Comentario de una sola línea (ej. // comentario)
    CommentMultiLine, // Comentario de múltiples líneas (ej. /* comentario */)
    Keyword,          // Palabra reservada del lenguaje (ej. let, if, while)
    Identifier,       // Nombre dado por el usuario a variables, funciones, etc.
    Integer,          // Número entero (ej. 10, 42)
    Float,            // Número de punto flotante (ej. 3.14, 0.5)
    String,           // Cadena de caracteres (ej. "hola", 'mundo')
    Plus,             // Operador de suma (+)
    Minus,            // Operador de resta (-)
    Asterisk,         // Operador de multiplicación (*)
    Slash,            // Operador de división (/)
    Equal,            // Operador de asignación o parte de igualdad (=)
    Greater,          // Operador mayor que (>)
    Less,             // Operador menor que (<)
    Exclamation,      // Signo de exclamación (!)
    Ampersand,        // Ampersand (&)
    Bar,              // Barra vertical (|)
    DoubleEqual,      // Operador de igualdad (==)
    GreaterEqual,     // Operador mayor o igual que (>=)
    LessEqual,        // Operador menor o igual que (<=)
    NotEqual,         // Operador de desigualdad (!= o <>)
    DoubleAmpersand,  // Operador lógico AND (&&)
    DoubleBar,        // Operador lógico OR (||)
    ArrowRight,       // Flecha (->)
    AtAsterisk,       // Arroba seguido de asterisco (@*)
    DotsPlus,         // Tres puntos seguidos de un más (...+)
    PipeGreater,      // Barra vertical seguida de mayor que (|>)
    Increment,        // Operador de incremento (++)
    Decrement,        // Operador de decremento (--)
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
    Whitespace,       // Espacio en blanco, tabulación, etc.
    NewLine,          // Salto de línea
    EndOfFile,        // Marcador de fin de archivo/entrada
    Invalid,          // Token no reconocido o inválido
    Unknown,          // Token desconocido
    Boolean,
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
