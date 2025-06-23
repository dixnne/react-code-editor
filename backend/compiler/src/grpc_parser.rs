use crate::ast::*;
use crate::parser::parse_tokens;
use crate::token::{LexerToken, TokenType};
use tonic::{Request, Response, Status};

pub mod lexer {
    tonic::include_proto!("lexer");
}

pub mod parser {
    tonic::include_proto!("parser");
}

use lexer::Token as ProtoToken;
use parser::{
    parser_server::{Parser},
    AstNode as ASTNode,
    ParseRequest, 
    ParseResponse, 
    ParserError,
    ParseSourceRequest,
};

#[derive(Debug, Default)]
pub struct ParserService;

#[tonic::async_trait]
impl Parser for ParserService {
    async fn parse(
        &self,
        request: Request<ParseRequest>,
    ) -> Result<Response<ParseResponse>, Status> {
        let proto_tokens = request.into_inner().tokens;

        let tokens: Vec<LexerToken> = proto_tokens
            .into_iter()
            .map(|t| LexerToken {
                token_type: TokenType::from_str(&t.token_type).unwrap_or(TokenType::Unknown),
                lexeme: t.lexeme,
                line: t.line as usize,
                column: t.column as usize,
            })
            .collect();

        let ParseResult { ast, errors } = parse_tokens(&tokens);

        Ok(Response::new(ParseResponse {
            ast: Some(program_to_proto(&ast)),
            errors: errors_to_proto(&errors),
        }))
    }

    async fn parse_source(
        &self,
        _request: Request<ParseSourceRequest>,
    ) -> Result<Response<ParseResponse>, Status> {
        Err(Status::unimplemented("parse_source no está implementado, usar parse con tokens."))
    }
}

// --- Funciones de Conversión de AST a Protobuf ---
// NOTA: Se añaden valores por defecto a los campos de posición.
// Idealmente, tu AST debería contener esta información y pasarla aquí.

fn program_to_proto(program: &Program) -> ASTNode {
    ASTNode {
        node_type: "Program".to_string(),
        value: "".to_string(),
        children: program.declarations.iter().map(declaration_to_proto).collect(),
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }
}

fn declaration_to_proto(decl: &Declaration) -> ASTNode {
    match decl {
        Declaration::Function(f) => function_to_proto(f),
        Declaration::Variable(v) => variable_decl_to_proto(v),
        Declaration::Struct(s) => struct_decl_to_proto(s),
        Declaration::Constant(c) => constant_decl_to_proto(c),
        Declaration::Statement(s) => statement_to_proto(s),
    }
}

fn constant_decl_to_proto(decl: &ConstantDeclaration) -> ASTNode {
    let mut children = vec![
        identifier_to_proto(&decl.identifier),
        expression_to_proto(&decl.value),
    ];
    if let Some(t) = &decl.const_type {
        children.insert(1, type_to_proto(t));
    }
    ASTNode {
        node_type: "ConstantDeclaration".to_string(),
        value: "const".to_string(),
        children,
        start_line: decl.identifier.line as u32, start_column: decl.identifier.column as u32, end_line: 0, end_column: 0,
    }
}

fn statement_to_proto(stmt: &Statement) -> ASTNode {
    match stmt {
        Statement::Expression(e) => expression_to_proto(e),
        Statement::Return(r) => return_stmt_to_proto(r),
        Statement::If(i) => if_stmt_to_proto(i),
        Statement::Block(b) => block_to_proto(b),
        Statement::While(w) => while_stmt_to_proto(w),
        Statement::For(f) => for_stmt_to_proto(f),
    }
}

fn block_to_proto(block: &Block) -> ASTNode {
    ASTNode {
        node_type: "Block".to_string(),
        value: "".to_string(),
        children: block.statements.iter().map(declaration_to_proto).collect(),
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }
}

fn expression_to_proto(expr: &Expression) -> ASTNode {
    match expr {
        Expression::Identifier(id) => identifier_to_proto(id),
        Expression::Literal(lit) => literal_to_proto(lit),
        Expression::Binary { left, op, right } => binary_expr_to_proto(left, op, right),
        Expression::Unary { op, expr } => unary_expr_to_proto(op, expr),
        Expression::Assignment { target, value } => assignment_to_proto(target, value),
        Expression::Postfix { expr, op } => postfix_expr_to_proto(expr, op),
        Expression::Grouped(expr) => grouped_expr_to_proto(expr),
        Expression::FunctionCall { function, arguments } => func_call_to_proto(function, arguments),
        Expression::Array(elements) => array_to_proto(elements),
        Expression::Object(fields) => object_to_proto(fields),
        Expression::Splat(expr) => splat_to_proto(expr),
        Expression::StructInstantiation { name, fields } => struct_inst_to_proto(name, fields),
    }
}

fn array_to_proto(elements: &[Expression]) -> ASTNode {
    ASTNode {
        node_type: "ArrayLiteral".to_string(),
        value: "".to_string(),
        children: elements.iter().map(expression_to_proto).collect(),
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }
}

fn object_to_proto(fields: &[(Identifier, Expression)]) -> ASTNode {
    let children = fields.iter().map(|(key, val)| ASTNode {
        node_type: "ObjectField".to_string(),
        value: "".to_string(),
        children: vec![identifier_to_proto(key), expression_to_proto(val)],
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }).collect();
    ASTNode {
        node_type: "ObjectLiteral".to_string(),
        value: "".to_string(),
        children,
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }
}

fn splat_to_proto(expr: &Expression) -> ASTNode {
    ASTNode {
        node_type: "Splat".to_string(),
        value: "@*".to_string(),
        children: vec![expression_to_proto(expr)],
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }
}

fn struct_inst_to_proto(name: &Identifier, fields: &[(Identifier, Expression)]) -> ASTNode {
     let field_children = fields.iter().map(|(key, val)| ASTNode {
        node_type: "StructFieldInit".to_string(),
        value: "".to_string(),
        children: vec![identifier_to_proto(key), expression_to_proto(val)],
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }).collect();
    ASTNode {
        node_type: "StructInstantiation".to_string(),
        value: name.name.clone(),
        children: field_children,
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }
}

fn binary_expr_to_proto(left: &Expression, op: &BinaryOp, right: &Expression) -> ASTNode {
    let op_str = match op {
        BinaryOp::Plus => "+", BinaryOp::Minus => "-", BinaryOp::Asterisk => "*",
        BinaryOp::Slash => "/", BinaryOp::Greater => ">", BinaryOp::Less => "<",
        BinaryOp::DoubleEqual => "==", BinaryOp::NotEqual => "!=",
        BinaryOp::DoubleAmpersand => "&&", BinaryOp::DoubleBar => "||",
        BinaryOp::GreaterEqual => ">=", BinaryOp::LessEqual => "<=",
        BinaryOp::Pipe => "|>", BinaryOp::Spread => "...+", BinaryOp::Swap => "<=>",
    };
    ASTNode {
        node_type: "Binary".to_string(), value: op_str.to_string(),
        children: vec![expression_to_proto(left), expression_to_proto(right)],
        start_line: 0, start_column: 0, end_line: 0, end_column: 0,
    }
}

fn identifier_to_proto(id: &Identifier) -> ASTNode {
    ASTNode {
        node_type: "Identifier".to_string(), value: id.name.clone(), children: vec![],
        start_line: id.line as u32, start_column: id.column as u32,
        end_line: id.line as u32, end_column: (id.column + id.name.len()) as u32,
    }
}

// ... El resto de las funciones de conversión también necesitan sus campos de posición
fn function_to_proto(func: &Function) -> ASTNode { unimplemented!() }
fn variable_decl_to_proto(decl: &VariableDeclaration) -> ASTNode { unimplemented!() }
fn struct_decl_to_proto(decl: &StructDeclaration) -> ASTNode { unimplemented!() }
fn return_stmt_to_proto(ret: &ReturnStatement) -> ASTNode { unimplemented!() }
fn if_stmt_to_proto(if_stmt: &IfStatement) -> ASTNode { unimplemented!() }
fn while_stmt_to_proto(while_stmt: &WhileStatement) -> ASTNode { unimplemented!() }
fn for_stmt_to_proto(for_stmt: &ForStatement) -> ASTNode { unimplemented!() }
fn literal_to_proto(lit: &Literal) -> ASTNode { unimplemented!() }
fn unary_expr_to_proto(op: &UnaryOp, expr: &Expression) -> ASTNode { unimplemented!() }
fn assignment_to_proto(target: &Identifier, value: &Expression) -> ASTNode { unimplemented!() }
fn postfix_expr_to_proto(expr: &Expression, op: &PostfixOp) -> ASTNode { unimplemented!() }
fn grouped_expr_to_proto(expr: &Expression) -> ASTNode { unimplemented!() }
fn func_call_to_proto(function: &Expression, arguments: &[Expression]) -> ASTNode { unimplemented!() }
fn type_to_proto(ty: &Type) -> ASTNode { unimplemented!() }
fn errors_to_proto(errors: &[SyntaxError]) -> Vec<ParserError> { unimplemented!() }
