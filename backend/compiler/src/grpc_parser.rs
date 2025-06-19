use crate::ast::*;
use crate::grpc_lexer::lexer;
use crate::grpc_parser::parser::ParseSourceRequest;
use crate::lexer::LexicalAnalyzer;
use crate::parser::parse_tokens;
use crate::token::{LexerToken, TokenType};
use tonic::{Request, Response, Status, transport::Server};

pub mod parser {
    tonic::include_proto!("parser");
}

use parser::{
    AstNode as ASTNode, ParseRequest, ParseResponse, ParserError,
    parser_server::{Parser, ParserServer},
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

        // Convert protobuf tokens to LexerToken
        let tokens: Vec<LexerToken> = proto_tokens
            .into_iter()
            .map(|t| LexerToken {
                token_type: TokenType::from_str(&t.token_type).unwrap_or(TokenType::Unknown),
                lexeme: t.lexeme,
                line: t.line as usize,
                column: t.column as usize,
            })
            .collect();

        // Parse tokens
        let ParseResult { ast, errors } = parse_tokens(&tokens);

        println!("Parsed AST: {:?}", ast);

        // Convert to protobuf response
        Ok(Response::new(ParseResponse {
            ast: Some(program_to_proto(&ast)),
            errors: errors_to_proto(&errors),
        }))
    }

    async fn parse_source(
        &self,
        request: Request<ParseSourceRequest>,
    ) -> Result<Response<ParseResponse>, Status> {
        let source = request.into_inner().source;

        // Call your lexer to get tokens
        let mut lexer = LexicalAnalyzer::new(&source);
        let tokens = lexer.scan_tokens(); // Assuming it returns Vec<LexerToken>

        // Filter out whitespace, newlines, and comments
        let filtered_tokens: Vec<LexerToken> = tokens
            .into_iter()
            .filter(|token| {
                !matches!(
                    token.token_type,
                    TokenType::Whitespace
                        | TokenType::NewLine
                        | TokenType::CommentSingle
                        | TokenType::CommentMultiLine
                )
            })
            .collect();

        // Parse tokens
        let ParseResult { ast, errors } = parse_tokens(&filtered_tokens);

        println!("Parsed AST: {:?}", ast);
        println!("Parse errors: {:?}", errors);

        // Convert to protobuf and respond
        Ok(Response::new(ParseResponse {
            ast: Some(program_to_proto(&ast)),
            errors: errors_to_proto(&errors),
        }))
    }
}

// ================= AST Conversion Functions =================
fn program_to_proto(program: &Program) -> ASTNode {
    ASTNode {
        node_type: "Program".to_string(),
        value: "".to_string(),
        children: program
            .declarations
            .iter()
            .map(declaration_to_proto)
            .collect(),
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn declaration_to_proto(decl: &Declaration) -> ASTNode {
    match decl {
        Declaration::Function(f) => function_to_proto(f),
        Declaration::Variable(v) => variable_decl_to_proto(v),
        Declaration::Struct(s) => struct_decl_to_proto(s),
    }
}

fn function_to_proto(func: &Function) -> ASTNode {
    let mut children = Vec::new();

    // Function name
    children.push(identifier_to_proto(&func.name));

    // Parameters
    let params_node = ASTNode {
        node_type: "Parameters".to_string(),
        value: "".to_string(),
        children: func.parameters.iter().map(parameter_to_proto).collect(),
        start_line: func.name.line as u32,
        start_column: func.name.column as u32,
        end_line: 0,
        end_column: 0,
    };
    children.push(params_node);

    // Return type
    children.push(type_to_proto(&func.return_type));

    // Body
    children.push(block_to_proto(&func.body));

    ASTNode {
        node_type: "Function".to_string(),
        value: "".to_string(),
        children,
        start_line: func.name.line as u32,
        start_column: func.name.column as u32,
        end_line: 0,
        end_column: 0,
    }
}

fn parameter_to_proto(param: &Parameter) -> ASTNode {
    let mut children = Vec::new();
    children.push(identifier_to_proto(&param.name));
    children.push(type_to_proto(&param.param_type));

    ASTNode {
        node_type: "Parameter".to_string(),
        value: "".to_string(),
        children,
        start_line: param.name.line as u32,
        start_column: param.name.column as u32,
        end_line: 0,
        end_column: 0,
    }
}

fn variable_decl_to_proto(decl: &VariableDeclaration) -> ASTNode {
    let mut children = Vec::new();

    // Identifier
    children.push(identifier_to_proto(&decl.identifier));

    // Type (if present)
    if let Some(t) = &decl.var_type {
        children.push(type_to_proto(t));
    }

    // Value
    children.push(expression_to_proto(&decl.value));

    ASTNode {
        node_type: "VariableDeclaration".to_string(),
        value: "".to_string(),
        children,
        start_line: decl.identifier.line as u32,
        start_column: decl.identifier.column as u32,
        end_line: 0,
        end_column: 0,
    }
}

fn struct_decl_to_proto(decl: &StructDeclaration) -> ASTNode {
    let mut children = Vec::new();

    // Struct name
    children.push(identifier_to_proto(&decl.name));

    // Fields
    let fields_node = ASTNode {
        node_type: "Fields".to_string(),
        value: "".to_string(),
        children: decl.fields.iter().map(field_decl_to_proto).collect(),
        start_line: decl.name.line as u32,
        start_column: decl.name.column as u32,
        end_line: 0,
        end_column: 0,
    };
    children.push(fields_node);

    ASTNode {
        node_type: "StructDeclaration".to_string(),
        value: "".to_string(),
        children,
        start_line: decl.name.line as u32,
        start_column: decl.name.column as u32,
        end_line: 0,
        end_column: 0,
    }
}

fn field_decl_to_proto(field: &FieldDeclaration) -> ASTNode {
    let mut children = Vec::new();
    children.push(identifier_to_proto(&field.name));
    children.push(type_to_proto(&field.field_type));

    ASTNode {
        node_type: "Field".to_string(),
        value: "".to_string(),
        children,
        start_line: field.name.line as u32,
        start_column: field.name.column as u32,
        end_line: 0,
        end_column: 0,
    }
}

fn statement_to_proto(stmt: &Statement) -> ASTNode {
    match stmt {
        Statement::VariableDeclaration(v) => variable_decl_to_proto(v),
        Statement::Expression(e) => expression_to_proto(e),
        Statement::Return(r) => return_stmt_to_proto(r),
        Statement::If(i) => if_stmt_to_proto(i),
        Statement::Block(b) => block_to_proto(b),
        Statement::While(w) => while_stmt_to_proto(w),
        Statement::For(f) => for_stmt_to_proto(f),
    }
}

fn return_stmt_to_proto(ret: &ReturnStatement) -> ASTNode {
    ASTNode {
        node_type: "Return".to_string(),
        value: "".to_string(),
        children: vec![expression_to_proto(&ret.value)],
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn if_stmt_to_proto(if_stmt: &IfStatement) -> ASTNode {
    let mut children = Vec::new();

    // Condition
    children.push(expression_to_proto(&if_stmt.condition));

    // Then block
    children.push(block_to_proto(&if_stmt.then_block));

    // Else block
    if let Some(else_branch) = &if_stmt.else_block {
        children.push(else_branch_to_proto(else_branch));
    }

    ASTNode {
        node_type: "If".to_string(),
        value: "".to_string(),
        children,
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn else_branch_to_proto(else_branch: &ElseBranch) -> ASTNode {
    match else_branch {
        ElseBranch::If(if_stmt) => if_stmt_to_proto(if_stmt),
        ElseBranch::Block(stmt) => {
            if let Statement::Block(ref block) = **stmt {
                block_to_proto(block)
            } else {
                panic!("Expected a Block statement in ElseBranch::Block");
            }
        }
    }
}

fn block_to_proto(block: &Block) -> ASTNode {
    ASTNode {
        node_type: "Block".to_string(),
        value: "".to_string(),
        children: block.statements.iter().map(statement_to_proto).collect(),
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn while_stmt_to_proto(while_stmt: &WhileStatement) -> ASTNode {
    ASTNode {
        node_type: "While".to_string(),
        value: "".to_string(),
        children: vec![
            expression_to_proto(&while_stmt.condition),
            block_to_proto(&while_stmt.body),
        ],
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn for_stmt_to_proto(for_stmt: &ForStatement) -> ASTNode {
    ASTNode {
        node_type: "For".to_string(),
        value: "".to_string(),
        children: vec![
            identifier_to_proto(&for_stmt.variable),
            expression_to_proto(&for_stmt.iterable),
            block_to_proto(&for_stmt.body),
        ],
        start_line: for_stmt.variable.line as u32,
        start_column: for_stmt.variable.column as u32,
        end_line: 0,
        end_column: 0,
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
        Expression::FunctionCall {
            function,
            arguments,
        } => func_call_to_proto(function, arguments),
    }
}

fn identifier_to_proto(id: &Identifier) -> ASTNode {
    ASTNode {
        node_type: "Identifier".to_string(),
        value: id.name.clone(),
        children: vec![],
        start_line: id.line as u32,
        start_column: id.column as u32,
        end_line: id.line as u32,
        end_column: (id.column + id.name.len()) as u32,
    }
}

fn literal_to_proto(lit: &Literal) -> ASTNode {
    let (value, node_type) = match lit {
        Literal::Int(i) => (i.to_string(), "IntLiteral"),
        Literal::Float(f) => (f.to_string(), "FloatLiteral"),
        Literal::String(s) => (s.clone(), "StringLiteral"),
        Literal::Bool(b) => (b.to_string(), "BoolLiteral"),
    };

    ASTNode {
        node_type: node_type.to_string(),
        value,
        children: vec![],
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn binary_expr_to_proto(left: &Expression, op: &BinaryOp, right: &Expression) -> ASTNode {
    let op_str = match op {
        BinaryOp::Plus => "+",
        BinaryOp::Minus => "-",
        BinaryOp::Asterisk => "*",
        BinaryOp::Slash => "/",
        BinaryOp::Greater => ">",
        BinaryOp::Less => "<",
        BinaryOp::DoubleEqual => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::DoubleAmpersand => "&&",
        BinaryOp::DoubleBar => "||",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::LessEqual => "<=",
    };

    ASTNode {
        node_type: "Binary".to_string(),
        value: op_str.to_string(),
        children: vec![expression_to_proto(left), expression_to_proto(right)],
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn unary_expr_to_proto(op: &UnaryOp, expr: &Expression) -> ASTNode {
    let op_str = match op {
        UnaryOp::Minus => "-",
        UnaryOp::Exclamation => "!",
    };

    ASTNode {
        node_type: "Unary".to_string(),
        value: op_str.to_string(),
        children: vec![expression_to_proto(expr)],
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn assignment_to_proto(target: &Identifier, value: &Expression) -> ASTNode {
    ASTNode {
        node_type: "Assignment".to_string(),
        value: "=".to_string(),
        children: vec![identifier_to_proto(target), expression_to_proto(value)],
        start_line: target.line as u32,
        start_column: target.column as u32,
        end_line: 0,
        end_column: 0,
    }
}

fn postfix_expr_to_proto(expr: &Expression, op: &PostfixOp) -> ASTNode {
    let op_str = match op {
        PostfixOp::Increment => "++",
        PostfixOp::Decrement => "--",
    };

    ASTNode {
        node_type: "Postfix".to_string(),
        value: op_str.to_string(),
        children: vec![expression_to_proto(expr)],
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn grouped_expr_to_proto(expr: &Expression) -> ASTNode {
    ASTNode {
        node_type: "Grouped".to_string(),
        value: "".to_string(),
        children: vec![expression_to_proto(expr)],
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn func_call_to_proto(function: &Expression, arguments: &[Expression]) -> ASTNode {
    let mut children = vec![expression_to_proto(function)];

    let args_node = ASTNode {
        node_type: "Arguments".to_string(),
        value: "".to_string(),
        children: arguments.iter().map(expression_to_proto).collect(),
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
    };
    children.push(args_node);

    ASTNode {
        node_type: "FunctionCall".to_string(),
        value: "".to_string(),
        children,
        start_line: 0, // Add actual positions
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn type_to_proto(ty: &Type) -> ASTNode {
    let type_str = match ty {
        Type::Int => "int",
        Type::Float => "float",
        Type::String => "string",
        Type::Bool => "bool",
    };

    ASTNode {
        node_type: "Type".to_string(),
        value: type_str.to_string(),
        children: vec![],
        start_line: 0,
        start_column: 0,
        end_line: 0,
        end_column: 0,
    }
}

fn errors_to_proto(errors: &[SyntaxError]) -> Vec<ParserError> {
    errors
        .iter()
        .map(|e| match e {
            SyntaxError::UnexpectedToken(msg, line, col) => ParserError {
                error_type: "UnexpectedToken".to_string(),
                message: msg.clone(),
                line: *line as u32,
                column: *col as u32,
            },
            SyntaxError::UnexpectedEndOfFile => ParserError {
                error_type: "UnexpectedEndOfFile".to_string(),
                message: "Unexpected end of file".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::InvalidAssignmentTarget => ParserError {
                error_type: "InvalidAssignmentTarget".to_string(),
                message: "Invalid assignment target".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::MissingSemicolon => ParserError {
                error_type: "MissingSemicolon".to_string(),
                message: "Missing semicolon".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::MissingColon => ParserError {
                error_type: "MissingColon".to_string(),
                message: "Missing colon".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::MissingType => ParserError {
                error_type: "MissingType".to_string(),
                message: "Missing type annotation".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::MissingInKeyword => ParserError {
                error_type: "MissingInKeyword".to_string(),
                message: "Missing 'in' keyword".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::MissingLoopVariable => ParserError {
                error_type: "MissingLoopVariable".to_string(),
                message: "Missing loop variable".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::MissingStructName => ParserError {
                error_type: "MissingStructName".to_string(),
                message: "Missing struct name".to_string(),
                line: 0,
                column: 0,
            },
            SyntaxError::MissingFieldName => ParserError {
                error_type: "MissingFieldName".to_string(),
                message: "Missing field name".to_string(),
                line: 0,
                column: 0,
            },
        })
        .collect()
}

// Add this to your TokenType implementation
impl TokenType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Keyword" => Some(TokenType::Keyword),
            "Identifier" => Some(TokenType::Identifier),
            "IntLiteral" => Some(TokenType::Integer),
            "FloatLiteral" => Some(TokenType::Float),
            "StringLiteral" => Some(TokenType::String),
            "BoolLiteral" => Some(TokenType::Boolean),
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
            "Ampersand" => Some(TokenType::Ampersand),
            "DoubleAmpersand" => Some(TokenType::DoubleAmpersand),
            "Bar" => Some(TokenType::Bar),
            "DoubleBar" => Some(TokenType::DoubleBar),
            "Exclamation" => Some(TokenType::Exclamation),
            "Comma" => Some(TokenType::Comma),
            "Semicolon" => Some(TokenType::Semicolon),
            "Colon" => Some(TokenType::Colon),
            "Dot" => Some(TokenType::Dot),
            "LeftParen" => Some(TokenType::LeftParen),
            "RightParen" => Some(TokenType::RightParen),
            "LeftBrace" => Some(TokenType::LeftBrace),
            "RightBrace" => Some(TokenType::RightBrace),
            "LeftBracket" => Some(TokenType::LeftBracket),
            "RightBracket" => Some(TokenType::RightBracket),
            _ => None,
        }
    }
}
