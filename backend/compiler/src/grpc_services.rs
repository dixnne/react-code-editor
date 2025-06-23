// Este archivo ahora sirve como el punto central para tus servicios gRPC.

// Usa el nombre del crate "compiler" con el prefijo `::` para una ruta absoluta.
use ::compiler::ast::*;
use ::compiler::lexer::LexicalAnalyzer;
use ::compiler::parser::parse_tokens;
use ::compiler::token::{LexerToken, TokenType};

// Imports de tonic y del código proto generado
use tonic::{Request, Response, Status};
pub mod compiler {
    tonic::include_proto!("compiler");
}
use compiler::{
    lexer_server::{Lexer, LexerServer}, // Se importan aquí aunque se usen en main
    parser_server::{Parser, ParserServer},
    AnalyzeRequest, AstNode, ParseRequest, ParseResponse, ParserError, Token, TokenList,
    ParseSourceRequest,
};

// --- Implementación del Servicio del Lexer ---

#[derive(Debug, Default)]
pub struct LexerService {}

#[tonic::async_trait]
impl Lexer for LexerService {
    async fn analyze(
        &self,
        request: Request<AnalyzeRequest>,
    ) -> std::result::Result<Response<TokenList>, Status> {
        let input_str = request.into_inner().input;
        let mut analyzer = LexicalAnalyzer::new(&input_str);
        let tokens = analyzer.scan_tokens();

        let token_list_proto = tokens
            .into_iter()
            .filter(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::NewLine | TokenType::CommentSingle | TokenType::CommentMultiLine | TokenType::Unknown))
            .map(|t| Token {
                token_type: t.token_type.to_string(),
                lexeme: t.lexeme,
                line: t.line as u32,
                column: t.column as u32,
            })
            .collect::<Vec<_>>();

        Ok(Response::new(TokenList { tokens: token_list_proto }))
    }
}

// --- Implementación del Servicio del Parser ---

#[derive(Debug, Default)]
pub struct ParserService;

#[tonic::async_trait]
impl Parser for ParserService {
    async fn parse(&self, request: Request<ParseRequest>) -> Result<Response<ParseResponse>, Status> {
        let proto_tokens = request.into_inner().tokens;
        let tokens: Vec<LexerToken> = proto_tokens
            .into_iter()
            .map(|t| LexerToken {
                token_type: TokenType::from_str(&t.token_type).unwrap_or(TokenType::Unknown),
                lexeme: t.lexeme, line: t.line as usize, column: t.column as usize,
            })
            .collect();

        let ParseResult { ast, errors } = parse_tokens(&tokens);
        Ok(Response::new(ParseResponse {
            ast: Some(program_to_proto(&ast)),
            errors: errors_to_proto(&errors),
        }))
    }
    
    async fn parse_source(&self, request: Request<ParseSourceRequest>) -> Result<Response<ParseResponse>, Status> {
        let source_code = request.into_inner().source;
        let mut lexer = LexicalAnalyzer::new(&source_code);
        let tokens = lexer.scan_tokens();
        let filtered_tokens: Vec<LexerToken> = tokens.into_iter().filter(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::NewLine | TokenType::CommentSingle | TokenType::CommentMultiLine | TokenType::Unknown)).collect();
        let ParseResult { ast, errors } = parse_tokens(&filtered_tokens);
        Ok(Response::new(ParseResponse {
            ast: Some(program_to_proto(&ast)),
            errors: errors_to_proto(&errors),
        }))
    }
}

// --- Funciones de Conversión de AST a Protobuf (Implementación Completa) ---

fn program_to_proto(program: &Program) -> AstNode {
    AstNode {
        node_type: "Program".to_string(), value: "".to_string(),
        children: program.declarations.iter().map(declaration_to_proto).collect(),
        ..Default::default()
    }
}

fn declaration_to_proto(decl: &Declaration) -> AstNode {
    match decl {
        Declaration::Function(f) => function_to_proto(f),
        Declaration::Variable(v) => variable_decl_to_proto(v),
        Declaration::Struct(s) => struct_decl_to_proto(s),
        Declaration::Constant(c) => constant_decl_to_proto(c),
        Declaration::Statement(s) => statement_to_proto(s),
    }
}

fn statement_to_proto(stmt: &Statement) -> AstNode {
    match stmt {
        Statement::Expression(e) => expression_to_proto(e),
        Statement::Return(r) => return_stmt_to_proto(r),
        Statement::If(i) => if_stmt_to_proto(i),
        Statement::Block(b) => block_to_proto(b),
        Statement::While(w) => while_stmt_to_proto(w),
        Statement::For(f) => for_stmt_to_proto(f),
    }
}

fn expression_to_proto(expr: &Expression) -> AstNode {
     match expr {
        Expression::Identifier(id) => identifier_to_proto(id),
        Expression::Literal(lit) => literal_to_proto(lit),
        Expression::Binary { left, op, right } => binary_expr_to_proto(left, op, right),
        Expression::Unary { op, expr } => unary_expr_to_proto(op, expr),
        Expression::Assignment { target, value } => assignment_to_proto(target, value),
        Expression::Grouped(expr) => grouped_expr_to_proto(expr),
        Expression::FunctionCall { function, arguments } => func_call_to_proto(function, arguments),
        Expression::Array(elements) => array_to_proto(elements),
        Expression::Object(fields) => object_to_proto(fields),
        Expression::Splat(expr) => splat_to_proto(expr),
        Expression::StructInstantiation { name, fields } => struct_inst_to_proto(name, fields),
        Expression::MemberAccess { object, property } => member_access_to_proto(object, property),
    }
}

fn member_access_to_proto(object: &Expression, property: &Identifier) -> AstNode {
    AstNode {
        node_type: "MemberAccess".to_string(), value: ".".to_string(),
        children: vec![expression_to_proto(object), identifier_to_proto(property)],
        ..Default::default()
    }
}

fn function_to_proto(func: &Function) -> AstNode {
    let params_node = AstNode {
        node_type: "Parameters".to_string(),
        children: func.parameters.iter().map(|p| {
            AstNode { 
                node_type: "Parameter".to_string(), 
                children: vec![identifier_to_proto(&p.name), type_to_proto(&p.param_type)], 
                ..Default::default() 
            }
        }).collect(),
        ..Default::default()
    };
    AstNode {
        node_type: "Function".to_string(), value: func.name.name.clone(),
        children: vec![params_node, type_to_proto(&func.return_type), block_to_proto(&func.body)],
        start_line: func.name.line as u32, start_column: func.name.column as u32,
        ..Default::default()
    }
}

fn variable_decl_to_proto(decl: &VariableDeclaration) -> AstNode {
    let mut children = vec![identifier_to_proto(&decl.identifier)];
    if let Some(t) = &decl.var_type { children.push(type_to_proto(t)); }
    children.push(expression_to_proto(&decl.value));
    AstNode {
        node_type: "VariableDeclaration".to_string(), value: "let".to_string(), children,
        start_line: decl.identifier.line as u32, start_column: decl.identifier.column as u32,
        ..Default::default()
    }
}

fn constant_decl_to_proto(decl: &ConstantDeclaration) -> AstNode {
    let mut children = vec![identifier_to_proto(&decl.identifier)];
    if let Some(t) = &decl.const_type { children.push(type_to_proto(t)); }
    children.push(expression_to_proto(&decl.value));
    AstNode {
        node_type: "ConstantDeclaration".to_string(), value: "const".to_string(), children,
        start_line: decl.identifier.line as u32, start_column: decl.identifier.column as u32,
        ..Default::default()
    }
}

fn struct_decl_to_proto(decl: &StructDeclaration) -> AstNode {
    let fields_node = AstNode {
        node_type: "Fields".to_string(),
        children: decl.fields.iter().map(|f| {
            AstNode {
                node_type: "Field".to_string(),
                children: vec![identifier_to_proto(&f.name), type_to_proto(&f.field_type)],
                ..Default::default()
            }
        }).collect(),
        ..Default::default()
    };
    AstNode {
        node_type: "StructDeclaration".to_string(), value: decl.name.name.clone(),
        children: vec![fields_node],
        start_line: decl.name.line as u32, start_column: decl.name.column as u32,
        ..Default::default()
    }
}

fn return_stmt_to_proto(ret: &ReturnStatement) -> AstNode {
    AstNode {
        node_type: "Return".to_string(),
        children: vec![expression_to_proto(&ret.value)],
        ..Default::default()
    }
}

fn if_stmt_to_proto(if_stmt: &IfStatement) -> AstNode {
    let mut children = vec![expression_to_proto(&if_stmt.condition), block_to_proto(&if_stmt.then_block)];
    if let Some(else_branch) = &if_stmt.else_block {
        let else_node = match else_branch {
            ElseBranch::If(nested_if) => if_stmt_to_proto(nested_if),
            ElseBranch::Block(block) => statement_to_proto(block),
        };
        children.push(AstNode { node_type: "Else".to_string(), children: vec![else_node], ..Default::default() });
    }
    AstNode { node_type: "If".to_string(), children, ..Default::default() }
}

fn while_stmt_to_proto(while_stmt: &WhileStatement) -> AstNode {
    AstNode {
        node_type: "While".to_string(),
        children: vec![expression_to_proto(&while_stmt.condition), block_to_proto(&while_stmt.body)],
        ..Default::default()
    }
}

fn for_stmt_to_proto(for_stmt: &ForStatement) -> AstNode {
    AstNode {
        node_type: "For".to_string(),
        children: vec![identifier_to_proto(&for_stmt.variable), expression_to_proto(&for_stmt.iterable), block_to_proto(&for_stmt.body)],
        start_line: for_stmt.variable.line as u32, start_column: for_stmt.variable.column as u32,
        ..Default::default()
    }
}

fn identifier_to_proto(id: &Identifier) -> AstNode {
    AstNode {
        node_type: "Identifier".to_string(), value: id.name.clone(),
        start_line: id.line as u32, start_column: id.column as u32,
        end_line: id.line as u32, end_column: (id.column + id.name.len()) as u32,
        ..Default::default()
    }
}

fn literal_to_proto(lit: &Literal) -> AstNode {
    let (value, node_type) = match lit {
        Literal::Int(i) => (i.to_string(), "IntLiteral"),
        Literal::Float(f) => (f.to_string(), "FloatLiteral"),
        Literal::String(s) => (s.clone(), "StringLiteral"),
        Literal::Bool(b) => (b.to_string(), "BoolLiteral"),
    };
    AstNode { node_type: node_type.to_string(), value, ..Default::default() }
}

fn type_to_proto(ty: &Type) -> AstNode {
    let type_str = match ty {
        Type::Int => "int", Type::Float => "float", Type::String => "string", Type::Bool => "bool", Type::Void => "void",
    };
    AstNode { node_type: "Type".to_string(), value: type_str.to_string(), ..Default::default() }
}

fn binary_expr_to_proto(left: &Expression, op: &BinaryOp, right: &Expression) -> AstNode {
    AstNode {
        node_type: "Binary".to_string(), value: format!("{:?}", op),
        children: vec![expression_to_proto(left), expression_to_proto(right)],
        ..Default::default()
    }
}

fn unary_expr_to_proto(op: &UnaryOp, expr: &Expression) -> AstNode {
    AstNode {
        node_type: "Unary".to_string(), value: format!("{:?}", op),
        children: vec![expression_to_proto(expr)],
        ..Default::default()
    }
}

// --- FUNCIÓN CORREGIDA ---
fn assignment_to_proto(target: &Expression, value: &Expression) -> AstNode {
    AstNode {
        node_type: "Assignment".to_string(), value: "=".to_string(),
        children: vec![expression_to_proto(target), expression_to_proto(value)],
        ..Default::default()
    }
}

fn grouped_expr_to_proto(expr: &Expression) -> AstNode {
    AstNode {
        node_type: "Grouped".to_string(),
        children: vec![expression_to_proto(expr)],
        ..Default::default()
    }
}

fn func_call_to_proto(function: &Expression, arguments: &[Expression]) -> AstNode {
    AstNode {
        node_type: "FunctionCall".to_string(),
        children: vec![expression_to_proto(function)].into_iter().chain(arguments.iter().map(expression_to_proto)).collect(),
        ..Default::default()
    }
}

fn array_to_proto(elements: &[Expression]) -> AstNode {
    AstNode {
        node_type: "ArrayLiteral".to_string(),
        children: elements.iter().map(expression_to_proto).collect(),
        ..Default::default()
    }
}

fn object_to_proto(fields: &[(Identifier, Expression)]) -> AstNode {
    AstNode {
        node_type: "ObjectLiteral".to_string(),
        children: fields.iter().map(|(key, val)| AstNode {
            node_type: "ObjectField".to_string(),
            children: vec![identifier_to_proto(key), expression_to_proto(val)],
            ..Default::default()
        }).collect(),
        ..Default::default()
    }
}

fn splat_to_proto(expr: &Expression) -> AstNode {
    AstNode {
        node_type: "Splat".to_string(), value: "@*".to_string(),
        children: vec![expression_to_proto(expr)],
        ..Default::default()
    }
}

fn struct_inst_to_proto(name: &Identifier, fields: &[(Identifier, Expression)]) -> AstNode {
     AstNode {
        node_type: "StructInstantiation".to_string(), value: name.name.clone(),
        children: fields.iter().map(|(key, val)| AstNode {
            node_type: "StructFieldInit".to_string(),
            children: vec![identifier_to_proto(key), expression_to_proto(val)],
            start_line: key.line as u32, start_column: key.column as u32,
            ..Default::default()
        }).collect(),
        start_line: name.line as u32, start_column: name.column as u32,
        ..Default::default()
    }
}

fn block_to_proto(block: &Block) -> AstNode {
    AstNode {
        node_type: "Block".to_string(),
        children: block.statements.iter().map(declaration_to_proto).collect(),
        ..Default::default()
    }
}

fn errors_to_proto(errors: &[SyntaxError]) -> Vec<ParserError> {
    errors.iter().map(|e| {
        let (error_type, message, line, column) = match e {
            SyntaxError::UnexpectedToken(msg, l, c) => ("UnexpectedToken", msg.clone(), *l, *c),
            _ => ("GenericError", format!("{}", e), 0, 0),
        };
        ParserError { error_type: error_type.to_string(), message, line: line as u32, column: column as u32, }
    }).collect()
}
