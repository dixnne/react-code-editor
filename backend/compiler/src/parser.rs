use crate::token::{TokenType, LexerToken};
use crate::ast::*;
pub struct Parser<'a> {
    tokens: &'a [LexerToken],
    current: usize,
    pub errors: Vec<SyntaxError>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [LexerToken]) -> Self {
        Parser {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> Option<&LexerToken> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&LexerToken> {
        if self.current > 0 {
            Some(&self.tokens[self.current - 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<&LexerToken> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek()
            .map_or(true, |t| t.token_type == TokenType::EndOfFile)
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        let peeked = self.peek();
        peeked.map_or(false, |t| t.token_type == token_type)
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: TokenType, error_msg: &str) -> Result<(), SyntaxError> {
        if self.match_token(token_type) {
            Ok(())
        } else if let Some(token) = self.peek() {
            Err(SyntaxError::UnexpectedToken(
                token.lexeme.clone(),
                token.line,
                token.column,
            ))
        } else {
            Err(SyntaxError::UnexpectedEndOfFile)
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if let Some(prev) = self.previous() {
                if prev.token_type == TokenType::Semicolon {
                    return;
                }
            }

            if let Some(next) = self.peek() {
                match next.token_type {
                    TokenType::Keyword => {
                        let keywords = ["fn", "let", "return", "if", "while", "for", "struct"];
                        if keywords.contains(&next.lexeme.as_str()) {
                            return;
                        }
                    }
                    TokenType::RightBrace => return,
                    _ => {}
                }
            }

            self.advance();
        }
    }

    pub fn parse(&mut self) -> Program {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(decl) => declarations.push(decl),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }

        Program { declarations }
    }

    fn declaration(&mut self) -> Result<Declaration, SyntaxError> {
        if self.match_token(TokenType::Keyword) {
            match self.previous().map(|t| t.lexeme.as_str()) {
                Some("fn") => self.function_declaration().map(Declaration::Function),
                Some("let") => self.variable_declaration().map(Declaration::Variable),
                Some("struct") => self.struct_declaration().map(Declaration::Struct),
                _ => {
                    let token = self.previous().unwrap();
                    Err(SyntaxError::UnexpectedToken(
                        token.lexeme.clone(),
                        token.line,
                        token.column,
                    ))
                }
            }
        } else {
            let token = self.peek().unwrap();
            Err(SyntaxError::UnexpectedToken(
                token.lexeme.clone(),
                token.line,
                token.column,
            ))
        }
    }

    fn function_declaration(&mut self) -> Result<Function, SyntaxError> {
        // Function name
        let name_token = if let Some(token) = self.advance() {
            if token.token_type == TokenType::Identifier {
                Identifier {
                    name: token.lexeme.clone(),
                    line: token.line,
                    column: token.column,
                }
            } else {
                return Err(SyntaxError::UnexpectedToken(
                    token.lexeme.clone(),
                    token.line,
                    token.column,
                ));
            }
        } else {
            return Err(SyntaxError::UnexpectedEndOfFile);
        };

        // Parameters
        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let parameters = self.parameters()?;
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        // Return type
        self.consume(TokenType::ArrowRight, "Expected '->' after parameters")?;
        let return_type = self.type_annotation()?;

        // Body
        let body = self.block_statement()?;

        Ok(Function {
            name: name_token,
            parameters,
            return_type,
            body,
        })
    }

    fn parameters(&mut self) -> Result<Vec<Parameter>, SyntaxError> {
        let mut params = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                let name_token = if let Some(token) = self.advance() {
                    if token.token_type == TokenType::Identifier {
                        Identifier {
                            name: token.lexeme.clone(),
                            line: token.line,
                            column: token.column,
                        }
                    } else {
                        return Err(SyntaxError::UnexpectedToken(
                            token.lexeme.clone(),
                            token.line,
                            token.column,
                        ));
                    }
                } else {
                    return Err(SyntaxError::UnexpectedEndOfFile);
                };

                self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.type_annotation()?;

                params.push(Parameter {
                    name: name_token,
                    param_type,
                });

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn type_annotation(&mut self) -> Result<Type, SyntaxError> {
        if let Some(token) = self.advance() {
            match token.lexeme.as_str() {
                "int" => Ok(Type::Int),
                "float" => Ok(Type::Float),
                "string" => Ok(Type::String),
                "bool" => Ok(Type::Bool),
                _ => Err(SyntaxError::UnexpectedToken(
                    token.lexeme.clone(),
                    token.line,
                    token.column,
                )),
            }
        } else {
            Err(SyntaxError::UnexpectedEndOfFile)
        }
    }

    fn variable_declaration(&mut self) -> Result<VariableDeclaration, SyntaxError> {
        // Variable name
        let name_token = if let Some(token) = self.advance() {
            if token.token_type == TokenType::Identifier {
                Identifier {
                    name: token.lexeme.clone(),
                    line: token.line,
                    column: token.column,
                }
            } else {
                return Err(SyntaxError::UnexpectedToken(
                    token.lexeme.clone(),
                    token.line,
                    token.column,
                ));
            }
        } else {
            return Err(SyntaxError::UnexpectedEndOfFile);
        };

        // Type annotation (optional)
        let var_type = if self.match_token(TokenType::Colon) {
            Some(self.type_annotation()?)
        } else {
            None
        };

        // Assignment
        self.consume(TokenType::Equal, "Expected '=' after variable name")?;
        let value = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;

        Ok(VariableDeclaration {
            identifier: name_token,
            var_type,
            value,
        })
    }

    fn block_statement(&mut self) -> Result<Block, SyntaxError> {
        self.consume(TokenType::LeftBrace, "Expected '{' before block")?;

        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            println!("Parsing statement in block...");
            match self.statement() {
                Ok(stmt) => {
                    statements.push(stmt);
                }
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize(); // skip tokens until a potential statement start
                    // After synchronization, let the loop retry parsing the next statement
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Block { statements })
    }

    fn statement(&mut self) -> Result<Statement, SyntaxError> {
        if self.check(TokenType::Keyword) {
            // Do not call match_token here yet, just peek
            let keyword = self.peek().unwrap().lexeme.as_str();

            match keyword {
                "let" => {
                    self.match_token(TokenType::Keyword); // consume it here
                    self.variable_declaration()
                        .map(Statement::VariableDeclaration)
                }
                "if" => {
                    self.match_token(TokenType::Keyword);
                    self.if_statement().map(Statement::If)
                }
                "while" => {
                    self.match_token(TokenType::Keyword);
                    self.while_statement().map(Statement::While)
                }
                "return" => {
                    self.match_token(TokenType::Keyword);
                    self.return_statement().map(Statement::Return)
                }
                "for" => {
                    self.match_token(TokenType::Keyword);
                    self.for_statement().map(Statement::For)
                }
                other => {
                    let token = self.peek().unwrap();
                    Err(SyntaxError::UnexpectedToken(
                        token.lexeme.clone(),
                        token.line,
                        token.column,
                    ))
                }
            }
        } else if self.check(TokenType::LeftBrace) {
            self.block_statement().map(Statement::Block)
        } else {
            let expr = self.expression()?;
            self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
            Ok(Statement::Expression(expr))
        }
    }

    fn return_statement(&mut self) -> Result<ReturnStatement, SyntaxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(ReturnStatement { value })
    }

    fn if_statement(&mut self) -> Result<IfStatement, SyntaxError> {
        // Consume the 'if' keyword (should be done before calling this method)
        // Or add: self.consume(TokenType::Keyword, "Expected 'if'")?;

        let condition = self.expression()?;
        let then_block = self.block_statement()?;

        let else_block = if self.check(TokenType::Keyword)
            && self.peek().map(|t| t.lexeme.as_str()) == Some("else")
        {
            self.advance(); // consume 'else'

            if self.check(TokenType::Keyword)
                && self.peek().map(|t| t.lexeme.as_str()) == Some("if")
            {
                // else if
                self.advance(); // consume 'if'
                Some(ElseBranch::If(Box::new(self.if_statement()?)))
            } else {
                // else block
                Some(ElseBranch::Block(Box::new(Statement::Block(
                    self.block_statement()?,
                ))))
            }
        } else {
            None
        };

        Ok(IfStatement {
            condition,
            then_block,
            else_block,
        })
    }

    fn expression(&mut self) -> Result<Expression, SyntaxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression, SyntaxError> {
        let expr = self.logical_or()?;

        if self.match_token(TokenType::Equal) {
            if let Expression::Identifier(target) = expr {
                let value = self.assignment()?;
                return Ok(Expression::Assignment {
                    target,
                    value: Box::new(value),
                });
            } else {
                let token = self.previous().unwrap();
                return Err(SyntaxError::InvalidAssignmentTarget);
            }
        }

        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.logical_and()?;

        while self.match_token(TokenType::DoubleBar) {
            let op = BinaryOp::DoubleBar;
            let right = self.logical_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.equality()?;

        while self.match_token(TokenType::DoubleAmpersand) {
            let op = BinaryOp::DoubleAmpersand;
            let right = self.equality()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.comparison()?;

        while self.match_token(TokenType::DoubleEqual) || self.match_token(TokenType::NotEqual) {
            let op = if self.previous().unwrap().token_type == TokenType::DoubleEqual {
                BinaryOp::DoubleEqual
            } else {
                BinaryOp::NotEqual
            };
            let right = self.comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.term()?;

        while self.match_token(TokenType::Greater)
            || self.match_token(TokenType::GreaterEqual)
            || self.match_token(TokenType::Less)
            || self.match_token(TokenType::LessEqual)
        {
            let op = match self.previous().unwrap().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual, // Fixed
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual, // Fixed
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.factor()?;

        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let op = if self.previous().unwrap().token_type == TokenType::Plus {
                BinaryOp::Plus
            } else {
                BinaryOp::Minus
            };
            let right = self.factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.unary()?;

        while self.match_token(TokenType::Asterisk) || self.match_token(TokenType::Slash) {
            let op = if self.previous().unwrap().token_type == TokenType::Asterisk {
                BinaryOp::Asterisk
            } else {
                BinaryOp::Slash
            };
            let right = self.unary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, SyntaxError> {
        if self.match_token(TokenType::Minus) || self.match_token(TokenType::Exclamation) {
            let op = if self.previous().unwrap().token_type == TokenType::Minus {
                UnaryOp::Minus
            } else {
                UnaryOp::Exclamation
            };
            let expr = self.unary()?;
            return Ok(Expression::Unary {
                op,
                expr: Box::new(expr),
            });
        }

        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(TokenType::Increment) {
                expr = Expression::Postfix {
                    expr: Box::new(expr),
                    op: PostfixOp::Increment,
                };
            } else if self.match_token(TokenType::Decrement) {
                expr = Expression::Postfix {
                    expr: Box::new(expr),
                    op: PostfixOp::Decrement,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expression) -> Result<Expression, SyntaxError> {
        let mut arguments = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

        Ok(Expression::FunctionCall {
            function: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expression, SyntaxError> {
        if self.match_token(TokenType::Integer) {
            let token = self.previous().unwrap();
            if let Ok(value) = token.lexeme.parse::<i64>() {
                return Ok(Expression::Literal(Literal::Int(value)));
            }
            Err(SyntaxError::UnexpectedToken(
                token.lexeme.clone(),
                token.line,
                token.column,
            ))
        } else if self.match_token(TokenType::Float) {
            let token = self.previous().unwrap();
            if let Ok(value) = token.lexeme.parse::<f64>() {
                return Ok(Expression::Literal(Literal::Float(value)));
            }
            Err(SyntaxError::UnexpectedToken(
                token.lexeme.clone(),
                token.line,
                token.column,
            ))
        } else if self.match_token(TokenType::String) {
            let token = self.previous().unwrap();
            Ok(Expression::Literal(Literal::String(token.lexeme.clone())))
        } else if self.match_token(TokenType::Identifier) {
            let token = self.previous().unwrap();
            Ok(Expression::Identifier(Identifier {
                name: token.lexeme.clone(),
                line: token.line,
                column: token.column,
            }))
        } else if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            Ok(Expression::Grouped(Box::new(expr)))
        } else {
            let token = self.peek().unwrap();
            Err(SyntaxError::UnexpectedToken(
                token.lexeme.clone(),
                token.line,
                token.column,
            ))
        }
    }

    fn while_statement(&mut self) -> Result<WhileStatement, SyntaxError> {
        // Condition expression
        let condition = self.expression()?;
        println!("{:#?}", condition);
        let body = self.block_statement()?;
        Ok(WhileStatement { condition, body })
    }

    fn for_statement(&mut self) -> Result<ForStatement, SyntaxError> {
        // Parse loop variable
        let var_token = self.advance().ok_or(SyntaxError::UnexpectedEndOfFile)?;
        let variable = if var_token.token_type == TokenType::Identifier {
            Identifier {
                name: var_token.lexeme.clone(),
                line: var_token.line,
                column: var_token.column,
            }
        } else {
            return Err(SyntaxError::UnexpectedToken(
                var_token.lexeme.clone(),
                var_token.line,
                var_token.column,
            ));
        };

        // FIX: Handle 'in' regardless of token type
        let in_token = self.advance().ok_or(SyntaxError::UnexpectedEndOfFile)?;
        if in_token.lexeme != "in" {
            return Err(SyntaxError::MissingInKeyword);
        }

        // Parse iterable expression
        let iterable = self.expression()?;
        let body = self.block_statement()?;

        Ok(ForStatement {
            variable,
            iterable,
            body,
        })
    }

    fn struct_declaration(&mut self) -> Result<StructDeclaration, SyntaxError> {
        // Struct name
        let name_token = self.advance().ok_or(SyntaxError::UnexpectedEndOfFile)?;
        let name = if name_token.token_type == TokenType::Identifier {
            Identifier {
                name: name_token.lexeme.clone(),
                line: name_token.line,
                column: name_token.column,
            }
        } else {
            return Err(SyntaxError::UnexpectedToken(
                name_token.lexeme.clone(),
                name_token.line,
                name_token.column,
            ));
        };

        // Fields
        self.consume(TokenType::LeftBrace, "Expected '{' after struct name")?;
        let mut fields = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            // Field name
            let field_name_token = self.advance().ok_or(SyntaxError::UnexpectedEndOfFile)?;
            let field_name = if field_name_token.token_type == TokenType::Identifier {
                Identifier {
                    name: field_name_token.lexeme.clone(),
                    line: field_name_token.line,
                    column: field_name_token.column,
                }
            } else {
                return Err(SyntaxError::UnexpectedToken(
                    field_name_token.lexeme.clone(),
                    field_name_token.line,
                    field_name_token.column,
                ));
            };

            // Colon
            self.consume(TokenType::Colon, "Expected ':' after field name")?;

            // Field type
            let field_type = self.type_annotation()?;
            fields.push(FieldDeclaration {
                name: field_name,
                field_type,
            });

            // Optional comma separator
            self.match_token(TokenType::Comma);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
        Ok(StructDeclaration { name, fields })
    }
}

pub fn parse_tokens(tokens: &[LexerToken]) -> ParseResult {
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    ParseResult {
        ast,
        errors: parser.errors,
    }
}