use core::fmt;

// --- Errores de Sintaxis ---
#[derive(Debug, PartialEq, Clone)] // Añadido `Clone` para un mejor manejo de errores
pub enum SyntaxError {
    UnexpectedToken(String, usize, usize),
    UnexpectedEndOfFile,
    InvalidAssignmentTarget,
    MissingSemicolon,
    MissingColon,
    MissingType,
    MissingInKeyword,
    MissingLoopVariable,
    MissingStructName,
    MissingFieldName,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxError::UnexpectedToken(t, line, col) => 
                write!(f, "Token inesperado '{}' en la línea {}, columna {}", t, line, col),
            SyntaxError::UnexpectedEndOfFile => 
                write!(f, "Final inesperado del archivo"),
            SyntaxError::InvalidAssignmentTarget => 
                write!(f, "El objetivo de la asignación no es válido"),
            SyntaxError::MissingSemicolon => 
                write!(f, "Falta punto y coma"),
            SyntaxError::MissingColon => 
                write!(f, "Faltan dos puntos"),
            SyntaxError::MissingType => 
                write!(f, "Falta anotación de tipo"),
            SyntaxError::MissingInKeyword => 
                write!(f, "Se esperaba la palabra clave 'in' en el bucle 'for'"),
            SyntaxError::MissingLoopVariable => 
                write!(f, "Se esperaba una variable en el bucle 'for'"),
            SyntaxError::MissingStructName => 
                write!(f, "Se esperaba un nombre de struct después de la palabra clave 'struct'"),
            SyntaxError::MissingFieldName => 
                write!(f, "Se esperaba un nombre de campo en la declaración del struct"),
        }
    }
}


// --- Tipos y Nodos del AST ---

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Void, 
}

#[derive(Debug, PartialEq, Clone)]
pub struct Identifier {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Assignment {
        target: Identifier,
        value: Box<Expression>,
    },
    Grouped(Box<Expression>),
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Array(Vec<Expression>),
    Object(Vec<(Identifier, Expression)>),
    Splat(Box<Expression>),
    StructInstantiation {
        name: Identifier,
        fields: Vec<(Identifier, Expression)>,
    },
    MemberAccess {
        object: Box<Expression>,
        property: Identifier,
    },
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    // Aritméticos
    Plus,
    Minus,
    Asterisk,
    Slash,
    // Relacionales
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    DoubleEqual,
    NotEqual,
    // Lógicos
    DoubleAmpersand,
    DoubleBar,
    // --- NUEVOS OPERADORES ---
    Pipe,   // |>
    Spread, // ...+
    Swap,   // <=>
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Minus,
    Exclamation,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Return(ReturnStatement),
    If(IfStatement),
    Block(Block),
    While(WhileStatement), 
    For(ForStatement),
    // --- NUEVA VARIANTE DE SENTENCIA ---
    DoUntil(DoUntilStatement),
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub statements: Vec<Declaration>, // Un bloque puede tener declaraciones y sentencias
}

#[derive(Debug, PartialEq)]
pub enum ElseBranch {
    If(Box<IfStatement>),
    Block(Box<Statement>),
}

#[derive(Debug, PartialEq)]
pub struct ReturnStatement {
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<ElseBranch>,
}

#[derive(Debug, PartialEq)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

// --- NUEVA ESTRUCTURA PARA DO-UNTIL ---
#[derive(Debug, PartialEq)]
pub struct DoUntilStatement {
    pub body: Block,
    pub condition: Expression,
}

#[derive(Debug, PartialEq)]
pub struct ForStatement {
    pub variable: Identifier,
    pub iterable: Expression,
    pub body: Block,
}

// --- Declaraciones de Alto Nivel ---

#[derive(Debug, PartialEq)]
pub enum Declaration {
    Function(Function),
    Variable(VariableDeclaration),
    Struct(StructDeclaration),
    Constant(ConstantDeclaration),
    Statement(Statement), 
}

#[derive(Debug, PartialEq)]
pub struct ConstantDeclaration {
    pub identifier: Identifier,
    pub const_type: Option<Type>,
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
    pub identifier: Identifier,
    pub var_type: Option<Type>,
    pub value: Expression,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: Identifier,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub name: Identifier,
    pub param_type: Type,
}

#[derive(Debug, PartialEq)]
pub struct StructDeclaration {
    pub name: Identifier,
    pub fields: Vec<FieldDeclaration>,
}

#[derive(Debug, PartialEq)]
pub struct FieldDeclaration {
    pub name: Identifier,
    pub field_type: Type,
}

// --- Raíz del AST y Resultado del Parseo ---

#[derive(Debug, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub struct ParseResult {
    pub ast: Program,
    pub errors: Vec<SyntaxError>,
}
