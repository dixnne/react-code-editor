use core::fmt;

#[derive(Debug, PartialEq)]
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
    // Add more error types as needed
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntaxError::UnexpectedToken(t, line, col) => 
                write!(f, "Unexpected token '{}' at line {}, column {}", t, line, col),
            SyntaxError::UnexpectedEndOfFile => 
                write!(f, "Unexpected end of file"),
            SyntaxError::InvalidAssignmentTarget => 
                write!(f, "Invalid assignment target"),
            SyntaxError::MissingSemicolon => 
                write!(f, "Missing semicolon"),
            SyntaxError::MissingColon => 
                write!(f, "Missing colon"),
            SyntaxError::MissingType => 
                write!(f, "Missing type annotation"),
            SyntaxError::MissingInKeyword => 
                write!(f, "Expected 'in' keyword in for loop"),
            SyntaxError::MissingLoopVariable => 
                write!(f, "Expected loop variable in for statement"),
            SyntaxError::MissingStructName => 
                write!(f, "Expected struct name after 'struct' keyword"),
            SyntaxError::MissingFieldName => 
                write!(f, "Expected field name in struct declaration"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    // Add more types as needed
}


#[derive(Debug, PartialEq)]
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
    Postfix {
        expr: Box<Expression>,
        op: PostfixOp,
    },
    Grouped(Box<Expression>),
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    }
}

#[derive(Debug, PartialEq)]
pub enum ElseBranch {
    If(Box<IfStatement>),
    Block(Box<Statement>),
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
    Plus,
    Minus,
    Asterisk,
    Slash,
    Greater,
    Less,
    DoubleEqual,
    NotEqual,
    DoubleAmpersand,
    DoubleBar,
    GreaterEqual,
    LessEqual
    // Add more binary operators
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Minus,
    Exclamation,
    // Add more unary operators
}

#[derive(Debug, PartialEq)]
pub enum PostfixOp {
    Increment,
    Decrement,
}

#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
    pub identifier: Identifier,
    pub var_type: Option<Type>,
    pub value: Expression,
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
pub struct Block {
    pub statements: Vec<Statement>,
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
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, PartialEq)]
pub struct ForStatement {
    pub variable: Identifier,
    pub iterable: Expression,
    pub body: Block,
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

#[derive(Debug, PartialEq)]
pub enum Declaration {
    Function(Function),
    Variable(VariableDeclaration),
    Struct(StructDeclaration),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    Expression(Expression),
    Return(ReturnStatement),
    If(IfStatement),
    Block(Block),
    While(WhileStatement), 
    For(ForStatement),
    // Add more statement types
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

#[derive(Debug)]
pub struct ParseResult {
    pub ast: Program,
    pub errors: Vec<SyntaxError>,
}
