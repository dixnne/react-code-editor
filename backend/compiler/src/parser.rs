use crate::ast::*;
use crate::token::{LexerToken, TokenType};

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

    // --- Métodos de Ayuda ---

    fn peek(&self) -> Option<&LexerToken> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&LexerToken> {
        self.current.checked_sub(1).map(|i| &self.tokens[i])
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
        self.peek().map_or(false, |t| t.token_type == token_type)
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn consume(&mut self, token_type: TokenType, error_msg: &str) -> Result<&LexerToken, SyntaxError> {
        if self.check(token_type) {
            Ok(self.advance().unwrap())
        } else if let Some(token) = self.peek() {
            let err = SyntaxError::UnexpectedToken(
                format!("{}, se encontró '{}'", error_msg, token.lexeme),
                token.line,
                token.column,
            );
            self.errors.push(err.clone());
            Err(err)
        } else {
            let err = SyntaxError::UnexpectedEndOfFile;
            self.errors.push(err.clone());
            Err(err)
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if let Some(prev) = self.previous() {
                if prev.token_type == TokenType::Semicolon { return; }
            }

            if let Some(next) = self.peek() {
                match next.lexeme.as_str() {
                    "fn" | "let" | "const" | "return" | "if" | "while" | "for" | "struct" => return,
                    _ => {}
                }
            }
            self.advance();
        }
    }
    
    // --- Lógica Principal del Parser ---

    pub fn parse(&mut self) -> Program {
        let mut declarations = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(decl) => declarations.push(decl),
                Err(_) => self.synchronize(),
            }
        }
        Program { declarations }
    }

    fn declaration(&mut self) -> Result<Declaration, SyntaxError> {
        if let Some(token) = self.peek() {
            if token.token_type == TokenType::Keyword {
                 match token.lexeme.as_str() {
                    "fn" => { self.advance(); return self.function_declaration().map(Declaration::Function); },
                    "let" => { self.advance(); return self.variable_declaration().map(Declaration::Variable); },
                    "const" => { self.advance(); return self.constant_declaration().map(Declaration::Constant); },
                    "struct" => { self.advance(); return self.struct_declaration().map(Declaration::Struct); },
                     _ => {}
                }
            }
        }
        self.statement().map(Declaration::Statement)
    }
    
    // --- Declaraciones ---

    fn function_declaration(&mut self) -> Result<Function, SyntaxError> {
        let name_token = self.consume(TokenType::Identifier, "Se esperaba un nombre de función.")?.clone();
        let name = Identifier { name: name_token.lexeme, line: name_token.line, column: name_token.column };
        self.consume(TokenType::LeftParen, "Se esperaba '(' después del nombre de función.")?;
        let parameters = self.parameters()?;
        self.consume(TokenType::RightParen, "Se esperaba ')' después de los parámetros.")?;
        self.consume(TokenType::ArrowRight, "Se esperaba '->' para el tipo de retorno.")?;
        let return_type = self.type_annotation()?;
        let body = self.block_statement()?;
        Ok(Function { name, parameters, return_type, body })
    }
    
    fn parameters(&mut self) -> Result<Vec<Parameter>, SyntaxError> {
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                let name_token = self.consume(TokenType::Identifier, "Se esperaba nombre de parámetro.")?.clone();
                let name = Identifier { name: name_token.lexeme, line: name_token.line, column: name_token.column };
                self.consume(TokenType::Colon, "Se esperaba ':' después del nombre del parámetro.")?;
                let param_type = self.type_annotation()?;
                params.push(Parameter { name, param_type });
                if !self.match_token(TokenType::Comma) { break; }
            }
        }
        Ok(params)
    }

    fn type_annotation(&mut self) -> Result<Type, SyntaxError> {
        if let Some(type_token) = self.peek() {
             match type_token.lexeme.as_str() {
                "int" => { self.advance(); Ok(Type::Int) },
                "float" => { self.advance(); Ok(Type::Float) },
                "string" => { self.advance(); Ok(Type::String) },
                "bool" => { self.advance(); Ok(Type::Bool) },
                "void" => { self.advance(); Ok(Type::Void) },
                _ => Err(SyntaxError::UnexpectedToken(format!("Tipo desconocido '{}'", type_token.lexeme), type_token.line, type_token.column)),
            }
        } else {
             Err(SyntaxError::UnexpectedEndOfFile)
        }
    }

    fn constant_declaration(&mut self) -> Result<ConstantDeclaration, SyntaxError> {
        let name_token = self.consume(TokenType::Identifier, "Se esperaba un nombre para la constante.")?.clone();
        let identifier = Identifier { name: name_token.lexeme, line: name_token.line, column: name_token.column };
        let const_type = if self.match_token(TokenType::Colon) { Some(self.type_annotation()?) } else { None };
        self.consume(TokenType::Equal, "Se esperaba '=' después del nombre de la constante.")?;
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Se esperaba ';' después de la declaración de la constante.")?;
        Ok(ConstantDeclaration { identifier, const_type, value })
    }

    fn variable_declaration(&mut self) -> Result<VariableDeclaration, SyntaxError> {
        let name_token = self.consume(TokenType::Identifier, "Se esperaba un nombre para la variable.")?.clone();
        let identifier = Identifier { name: name_token.lexeme, line: name_token.line, column: name_token.column };
        let var_type = if self.match_token(TokenType::Colon) { Some(self.type_annotation()?) } else { None };
        self.consume(TokenType::Equal, "Se esperaba '=' en la declaración de la variable.")?;
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Se esperaba ';' después de la declaración de la variable.")?;
        Ok(VariableDeclaration { identifier, var_type, value })
    }
    
    fn struct_declaration(&mut self) -> Result<StructDeclaration, SyntaxError> {
        let name = self.consume(TokenType::Identifier, "Se esperaba un nombre para el struct.")?.clone();
        let name_id = Identifier { name: name.lexeme, line: name.line, column: name.column };
        self.consume(TokenType::LeftBrace, "Se esperaba '{' después del nombre del struct.")?;
        let mut fields = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let field_name_token = self.consume(TokenType::Identifier, "Se esperaba un nombre de campo.")?.clone();
            let field_name = Identifier { name: field_name_token.lexeme, line: field_name_token.line, column: field_name_token.column };
            self.consume(TokenType::Colon, "Se esperaba ':' después del nombre de campo.")?;
            let field_type = self.type_annotation()?;
            fields.push(FieldDeclaration { name: field_name, field_type });
            if !self.check(TokenType::RightBrace) {
                if !self.match_token(TokenType::Comma) {
                     let err = self.peek().unwrap();
                     return Err(SyntaxError::UnexpectedToken(format!("Se esperaba ',' o '}}' después del campo de struct, se encontró '{}'", err.lexeme), err.line, err.column));
                }
            }
        }
        self.consume(TokenType::RightBrace, "Se esperaba '}' al final del struct.")?;
        Ok(StructDeclaration { name: name_id, fields })
    }
    
    // --- Sentencias ---
    
    fn statement(&mut self) -> Result<Statement, SyntaxError> {
        if let Some(token) = self.peek() {
            if token.token_type == TokenType::Keyword {
                 match token.lexeme.as_str() {
                    "if" => { self.advance(); return self.if_statement().map(Statement::If); },
                    "while" => { self.advance(); return self.while_statement().map(Statement::While); },
                    "return" => { self.advance(); return self.return_statement().map(Statement::Return); },
                    "for" => { self.advance(); return self.for_statement().map(Statement::For); },
                    _ => {}
                }
            }
        }
        if self.check(TokenType::LeftBrace) {
            return self.block_statement().map(Statement::Block);
        }
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Se esperaba ';' después de la expresión.")?;
        Ok(Statement::Expression(expr))
    }
    
    fn block_statement(&mut self) -> Result<Block, SyntaxError> {
        self.consume(TokenType::LeftBrace, "Se esperaba '{' para iniciar un bloque.")?;
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(decl) => statements.push(decl),
                Err(_) => self.synchronize(),
            }
        }
        self.consume(TokenType::RightBrace, "Se esperaba '}' para cerrar un bloque.")?;
        Ok(Block { statements })
    }

    fn return_statement(&mut self) -> Result<ReturnStatement, SyntaxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Se esperaba ';' después del valor de retorno.")?;
        Ok(ReturnStatement { value })
    }

    fn logical_and(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.equality()?;
        while self.match_token(TokenType::DoubleAmpersand) {
            let op = BinaryOp::DoubleAmpersand;
            let right = self.equality()?;
            expr = Expression::Binary { 
                left: Box::new(expr), 
                op, 
                right: Box::new(right) 
            };
        }
        Ok(expr)
    }

    fn if_statement(&mut self) -> Result<IfStatement, SyntaxError> {
        let condition = self.expression()?;
        let then_block = self.block_statement()?;
        let mut else_block = None;
        if self.peek().map_or(false, |t| t.lexeme == "else") {
            self.advance();
            if self.peek().map_or(false, |t| t.lexeme == "if") {
                self.advance();
                else_block = Some(ElseBranch::If(Box::new(self.if_statement()?)));
            } else {
                else_block = Some(ElseBranch::Block(Box::new(Statement::Block(self.block_statement()?))));
            }
        }
        Ok(IfStatement { condition, then_block, else_block })
    }

    fn while_statement(&mut self) -> Result<WhileStatement, SyntaxError> {
        let condition = self.expression()?;
        let body = self.block_statement()?;
        Ok(WhileStatement { condition, body })
    }

    fn for_statement(&mut self) -> Result<ForStatement, SyntaxError> {
        let variable_token = self.consume(TokenType::Identifier, "Se esperaba una variable de bucle después de 'for'.")?.clone();
        let variable = Identifier { name: variable_token.lexeme, line: variable_token.line, column: variable_token.column };
        if let Some(token) = self.peek() {
            if token.token_type == TokenType::Keyword && token.lexeme == "in" {
                self.advance();
            } else {
                return Err(SyntaxError::MissingInKeyword);
            }
        } else {
            return Err(SyntaxError::UnexpectedEndOfFile);
        }
        let iterable = self.expression()?;
        let body = self.block_statement()?;
        Ok(ForStatement { variable, iterable, body })
    }

    // --- Expresiones y Jerarquía de Precedencia ---

    fn expression(&mut self) -> Result<Expression, SyntaxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression, SyntaxError> {
        let left = self.pipe()?;
        if self.match_token(TokenType::Equal) {
            if matches!(&left, Expression::Identifier(_) | Expression::MemberAccess { .. }) {
                let value = self.assignment()?;
                return Ok(Expression::Assignment {
                    target: Box::new(left),
                    value: Box::new(value),
                });
            }
            return Err(SyntaxError::InvalidAssignmentTarget);
        } else if self.match_token(TokenType::Swap) {
            if let Expression::Identifier(_) = &left {
                let right = self.assignment()?;
                if let Expression::Identifier(_) = &right {
                     return Ok(Expression::Binary { left: Box::new(left), op: BinaryOp::Swap, right: Box::new(right) });
                }
            }
            return Err(SyntaxError::InvalidAssignmentTarget);
        }
        Ok(left)
    }

    fn pipe(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.spread()?;
        while self.match_token(TokenType::Pipe) {
            let op = BinaryOp::Pipe;
            let right = self.spread()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }
    
    fn spread(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.logical_or()?;
        while self.match_token(TokenType::Spread) {
            let op = BinaryOp::Spread;
            let right = self.logical_or()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.logical_and()?;
        while self.match_token(TokenType::DoubleBar) {
            let op = BinaryOp::DoubleBar;
            let right = self.logical_and()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.equality()?;
        while self.match_token(TokenType::DoubleAmpersand) {
            let op = BinaryOp::DoubleAmpersand;
            let right = self.equality()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.comparison()?;
        while self.match_token(TokenType::DoubleEqual) || self.match_token(TokenType::NotEqual) {
            let op = if self.previous().unwrap().token_type == TokenType::DoubleEqual { BinaryOp::DoubleEqual } else { BinaryOp::NotEqual };
            let right = self.comparison()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.term()?;
        while self.match_token(TokenType::Greater) || self.match_token(TokenType::GreaterEqual) || self.match_token(TokenType::Less) || self.match_token(TokenType::LessEqual) {
            let op = match self.previous().unwrap().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.factor()?;
        while self.match_token(TokenType::Plus) || self.match_token(TokenType::Minus) {
            let op = if self.previous().unwrap().token_type == TokenType::Plus { BinaryOp::Plus } else { BinaryOp::Minus };
            let right = self.factor()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.unary()?;
        while self.match_token(TokenType::Asterisk) || self.match_token(TokenType::Slash) {
            let op = if self.previous().unwrap().token_type == TokenType::Asterisk { BinaryOp::Asterisk } else { BinaryOp::Slash };
            let right = self.unary()?;
            expr = Expression::Binary { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression, SyntaxError> {
        if self.match_token(TokenType::Minus) || self.match_token(TokenType::Exclamation) {
            let op = if self.previous().unwrap().token_type == TokenType::Minus { UnaryOp::Minus } else { UnaryOp::Exclamation };
            let expr = self.unary()?;
            return Ok(Expression::Unary { op, expr: Box::new(expr) });
        } else if self.match_token(TokenType::Splat) {
            let expr = self.unary()?;
            return Ok(Expression::Splat(Box::new(expr)));
        }
        self.postfix()
    }
    
    fn postfix(&mut self) -> Result<Expression, SyntaxError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(TokenType::LeftBrace) {
                expr = self.finish_struct_instantiation(expr)?;
            }
            else if self.match_token(TokenType::Dot) {
                let property = self.consume(TokenType::Identifier, "Se esperaba el nombre de la propiedad después de '.'.")?;
                expr = Expression::MemberAccess {
                    object: Box::new(expr),
                    property: Identifier {
                        name: property.lexeme.clone(),
                        line: property.line,
                        column: property.column,
                    },
                };
            } else if self.match_token(TokenType::Increment) || self.match_token(TokenType::Decrement) {
                if matches!(&expr, Expression::Identifier(_) | Expression::MemberAccess { .. }) {
                    let op_type = self.previous().unwrap().token_type;
                    let binary_op = if op_type == TokenType::Increment { BinaryOp::Plus } else { BinaryOp::Minus };
                    
                    let right_hand_side = Expression::Binary {
                        left: Box::new(expr.clone()),
                        op: binary_op,
                        right: Box::new(Expression::Literal(Literal::Int(1))),
                    };
                    
                    expr = Expression::Assignment {
                        target: Box::new(expr),
                        value: Box::new(right_hand_side),
                    };
                } else {
                    return Err(SyntaxError::InvalidAssignmentTarget);
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }
    
    fn finish_struct_instantiation(&mut self, callee: Expression) -> Result<Expression, SyntaxError> {
        let name = if let Expression::Identifier(id) = callee {
            id
        } else {
            return Err(SyntaxError::InvalidAssignmentTarget);
        };
        let mut fields = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let key_token = self.consume(TokenType::Identifier, "Se esperaba un nombre de campo.")?.clone();
            let key = Identifier { name: key_token.lexeme, line: key_token.line, column: key_token.column };
            self.consume(TokenType::Equal, "Se esperaba '=' después del nombre del campo.")?;
            let value = self.expression()?;
            fields.push((key, value));
            if !self.check(TokenType::RightBrace) {
               if !self.match_token(TokenType::Comma) {
                    return Err(SyntaxError::UnexpectedToken("Se esperaba ',' o '}' en la instanciación del struct.".to_string(), 0, 0));
               }
            }
        }
        self.consume(TokenType::RightBrace, "Se esperaba '}' al final de la instanciación.")?;
        Ok(Expression::StructInstantiation { name, fields })
    }

    fn primary(&mut self) -> Result<Expression, SyntaxError> {
        if self.match_token(TokenType::Integer) {
            let token = self.previous().unwrap();
            return Ok(Expression::Literal(Literal::Int(token.lexeme.parse().unwrap())));
        }
        if self.match_token(TokenType::Float) {
            let token = self.previous().unwrap();
            return Ok(Expression::Literal(Literal::Float(token.lexeme.parse().unwrap())));
        }
        if self.match_token(TokenType::String) {
            let token = self.previous().unwrap();
            return Ok(Expression::Literal(Literal::String(token.lexeme.clone())));
        }
        if self.match_token(TokenType::LeftBracket) {
            let mut elements = Vec::new();
            if !self.check(TokenType::RightBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(TokenType::Comma) { break; }
                }
            }
            self.consume(TokenType::RightBracket, "Se esperaba ']' al final del array.")?;
            return Ok(Expression::Array(elements));
        }
        
        if self.match_token(TokenType::Identifier) {
            let token = self.previous().unwrap();
            return Ok(Expression::Identifier(Identifier {
                name: token.lexeme.clone(),
                line: token.line,
                column: token.column,
            }));
        }

        if self.match_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Se esperaba ')' después de la expresión.")?;
            return Ok(Expression::Grouped(Box::new(expr)));
        }

        let token = self.peek().unwrap();
        Err(SyntaxError::UnexpectedToken(format!("Token inesperado: '{}'", token.lexeme), token.line, token.column))
    }

    fn finish_call(&mut self, callee: Expression) -> Result<Expression, SyntaxError> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(TokenType::Comma) { break; }
            }
        }
        self.consume(TokenType::RightParen, "Se esperaba ')' después de los argumentos.")?;
        Ok(Expression::FunctionCall { function: Box::new(callee), arguments })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::LexicalAnalyzer;
    use crate::token::TokenType;
    
    fn run_parser_with_errors(source: &str) -> ParseResult {
        let mut lexer = LexicalAnalyzer::new(source);
        let tokens = lexer.scan_tokens();
        let filtered_tokens: Vec<LexerToken> = tokens.into_iter().filter(|t| !matches!(t.token_type, TokenType::Whitespace | TokenType::NewLine | TokenType::CommentSingle | TokenType::CommentMultiLine | TokenType::Unknown)).collect();
        parse_tokens(&filtered_tokens)
    }

    #[test]
    fn test_for_loop_parsing() {
        let source = "for item in items { }";
        let result = run_parser_with_errors(source);
        assert!(result.errors.is_empty(), "No debería haber errores al analizar un bucle for válido. Errores: {:?}", result.errors);
        
        let decl = &result.ast.declarations[0];
        if let Declaration::Statement(Statement::For(_)) = decl {
            // La prueba pasa si se reconoce un ForStatement
        } else {
            panic!("No se analizó un ForStatement correctamente.");
        }
    }
}
