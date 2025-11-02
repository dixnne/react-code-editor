use crate::ast::*;
use crate::grpc_services::compiler::AnnotatedNode;
use crate::symbol_table::{Symbol, SymbolTable};

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticError {
    UndeclaredVariable(String, usize, usize),
    RedeclaredVariable(String, usize, usize),
    TypeMismatch(String, String, usize, usize),
    InvalidAssignment(String, usize, usize),
    UndefinedStruct(String, usize, usize),
    RedeclaredStruct(String, usize, usize),
    RedeclaredField(String, String, usize, usize),
    FieldNotFound(String, String, usize, usize),
    InvalidMemberAccess(String, usize, usize),
    InvalidFunctionCallTarget(usize, usize),
    UndefinedFunction(String, usize, usize),
    ArgumentCountMismatch(String, usize, usize, usize, usize),
    ArgumentTypeMismatch(String, usize, String, String, usize, usize),
    ReturnOutsideFunction(usize, usize),
    ReturnTypeMismatch(String, String, usize, usize),
    MissingReturnStatement(String, usize, usize),
    MissingMainFunction,
    InvalidMainFunctionSignature(String, usize, usize),
}

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub errors: Vec<SemanticError>,
    current_function: Option<(String, Type)>, // (function name, return type)
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            current_function: None,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> AnnotatedNode {
        let children = program
            .declarations
            .iter()
            .map(|d| self.analyze_declaration(d))
            .collect();
        self.check_for_main_function();
        AnnotatedNode {
            node_type: "Program".to_string(),
            children,
            ..Default::default()
        }
    }

    fn check_for_main_function(&mut self) {
        match self.symbol_table.lookup("main") {
            Some(symbol) => {
                if let Symbol::Function {
                    parameters,
                    return_type,
                    line,
                    column,
                    ..
                } = symbol
                {
                    let mut signature_errors = Vec::new();
                    if !parameters.is_empty() {
                        signature_errors.push(format!(
                            "expected 0 parameters but found {}",
                            parameters.len()
                        ));
                    }
                    if *return_type != Type::Int {
                        signature_errors.push(format!(
                            "expected a 'Int' return type but found '{}'",
                            return_type.to_string()
                        ));
                    }
                    if !signature_errors.is_empty() {
                        let reason = format!(
                            "Invalid 'main' function signature: {}",
                            signature_errors.join(" and ")
                        );
                        self.errors
                            .push(SemanticError::InvalidMainFunctionSignature(
                                reason, *line, *column,
                            ));
                    }
                } else {
                    self.errors.push(SemanticError::MissingMainFunction);
                }
            }
            None => {
                self.errors.push(SemanticError::MissingMainFunction);
            }
        }
    }

    fn analyze_declaration(&mut self, declaration: &Declaration) -> AnnotatedNode {
        match declaration {
            Declaration::Variable(var_decl) => self.analyze_variable_declaration(var_decl),
            Declaration::Function(func_decl) => self.analyze_function_declaration(func_decl),
            Declaration::Struct(struct_decl) => self.analyze_struct_declaration(struct_decl),
            Declaration::Constant(const_decl) => self.analyze_constant_declaration(const_decl),
            Declaration::Statement(stmt) => self.analyze_statement(stmt),
        }
    }

    fn analyze_variable_declaration(&mut self, var_decl: &VariableDeclaration) -> AnnotatedNode {
        let name = &var_decl.identifier.name;
        let declared_type = self.get_type(&var_decl.var_type);
        let value_node = self.analyze_expression(&var_decl.value);
        let value_type = Type::from_str(&value_node.inferred_type).unwrap_or(Type::Void);

        if declared_type != Type::Void && declared_type != value_type {
            self.errors.push(SemanticError::TypeMismatch(
                declared_type.to_string(),
                value_type.to_string(),
                var_decl.identifier.line,
                var_decl.identifier.column,
            ));
        }

        let literal_value = if let Expression::Literal(lit) = &var_decl.value {
            Some(lit.clone())
        } else {
            None
        };

        let symbol = Symbol::Variable {
            name: name.clone(),
            type_: value_type.clone(),
            defined: true,
            line: var_decl.identifier.line,
            column: var_decl.identifier.column,
            value: literal_value,
        };
        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(
                name.clone(),
                var_decl.identifier.line,
                var_decl.identifier.column,
            ));
        }

        AnnotatedNode {
            node_type: "VariableDeclaration".to_string(),
            value: "let".to_string(),
            children: vec![self.identifier_to_annotated(&var_decl.identifier), value_node],
            start_line: var_decl.identifier.line as u32,
            start_column: var_decl.identifier.column as u32,
            inferred_type: value_type.to_string(),
            ..Default::default()
        }
    }

    fn analyze_constant_declaration(&mut self, const_decl: &ConstantDeclaration) -> AnnotatedNode {
        let name = &const_decl.identifier.name;
        let declared_type = self.get_type(&const_decl.const_type);
        let value_node = self.analyze_expression(&const_decl.value);
        let value_type = Type::from_str(&value_node.inferred_type).unwrap_or(Type::Void);

        if declared_type != Type::Void && declared_type != value_type {
            self.errors.push(SemanticError::TypeMismatch(
                declared_type.to_string(),
                value_type.to_string(),
                const_decl.identifier.line,
                const_decl.identifier.column,
            ));
        }

        let literal_value = if let Expression::Literal(lit) = &const_decl.value {
            Some(lit.clone())
        } else {
            None
        };

        let symbol = Symbol::Constant {
            name: name.clone(),
            type_: value_type.clone(),
            line: const_decl.identifier.line,
            column: const_decl.identifier.column,
            value: literal_value,
        };

        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(
                name.clone(),
                const_decl.identifier.line,
                const_decl.identifier.column,
            ));
        }

        AnnotatedNode {
            node_type: "ConstantDeclaration".to_string(),
            value: "const".to_string(),
            children: vec![
                self.identifier_to_annotated(&const_decl.identifier),
                value_node,
            ],
            start_line: const_decl.identifier.line as u32,
            start_column: const_decl.identifier.column as u32,
            inferred_type: value_type.to_string(),
            ..Default::default()
        }
    }

    fn analyze_function_declaration(&mut self, func_decl: &Function) -> AnnotatedNode {
        let name = &func_decl.name.name;
        let parameters: Vec<Type> = func_decl
            .parameters
            .iter()
            .map(|p| p.param_type.clone())
            .collect();
        let return_type = func_decl.return_type.clone();
        let symbol = Symbol::Function {
            name: name.clone(),
            parameters: parameters.clone(),
            return_type: return_type.clone(),
            line: func_decl.name.line,
            column: func_decl.name.column,
        };

        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(
                name.clone(),
                func_decl.name.line,
                func_decl.name.column,
            ));
        }

        let previous_function = self.current_function.take();
        self.current_function = Some((name.clone(), return_type.clone()));

        self.symbol_table.enter_scope(format!("function: {}", name));
        let params_nodes: Vec<AnnotatedNode> = func_decl
            .parameters
            .iter()
            .map(|p| {
                let param_name = &p.name.name;
                let param_symbol = Symbol::Variable {
                    name: param_name.clone(),
                    type_: p.param_type.clone(),
                    defined: true,
                    line: p.name.line,
                    column: p.name.column,
                    value: None,
                };
                if !self.symbol_table.insert(param_name.clone(), param_symbol) {
                    self.errors.push(SemanticError::RedeclaredVariable(
                        param_name.clone(),
                        p.name.line,
                        p.name.column,
                    ));
                }
                AnnotatedNode {
                    node_type: "Parameter".to_string(),
                    value: p.name.name.clone(),
                    inferred_type: p.param_type.to_string(),
                    start_line: p.name.line as u32,
                    start_column: p.name.column as u32,
                    ..Default::default()
                }
            })
            .collect();

        let mut has_return = false;
        let body_node = self.analyze_block_with_return_check(&func_decl.body, &mut has_return);

        if return_type != Type::Void && !has_return {
            self.errors.push(SemanticError::MissingReturnStatement(
                name.clone(),
                func_decl.name.line,
                func_decl.name.column,
            ));
        }

        self.symbol_table.leave_scope();
        self.current_function = previous_function;

        AnnotatedNode {
            node_type: "FunctionDeclaration".to_string(),
            value: name.clone(),
            children: vec![
                AnnotatedNode {
                    node_type: "Parameters".to_string(),
                    children: params_nodes,
                    ..Default::default()
                },
                body_node,
            ],
            start_line: func_decl.name.line as u32,
            start_column: func_decl.name.column as u32,
            inferred_type: return_type.to_string(),
            ..Default::default()
        }
    }

    fn analyze_block_with_return_check(
        &mut self,
        block: &Block,
        has_return: &mut bool,
    ) -> AnnotatedNode {
        self.symbol_table.enter_scope("block".to_string());
        let mut children = vec![];
        for decl in &block.statements {
            if let Declaration::Statement(stmt) = decl {
                children.push(self.analyze_statement_with_return_check(stmt, has_return));
            } else {
                children.push(self.analyze_declaration(decl));
            }
        }
        self.symbol_table.leave_scope();
        AnnotatedNode {
            node_type: "Block".to_string(),
            children,
            ..Default::default()
        }
    }

    fn analyze_statement_with_return_check(
        &mut self,
        stmt: &Statement,
        has_return: &mut bool,
    ) -> AnnotatedNode {
        match stmt {
            Statement::Return(r) => {
                *has_return = true;
                self.analyze_return_statement(r)
            }
            Statement::Block(block) => self.analyze_block_with_return_check(block, has_return),
            Statement::If(if_stmt) => {
                let then_node = self.analyze_statement_with_return_check(
                    &Statement::Block(if_stmt.then_block.clone()),
                    has_return,
                );
                let else_node = if let Some(else_branch) = &if_stmt.else_block {
                    match else_branch {
                        ElseBranch::Block(block) => {
                            Some(self.analyze_statement_with_return_check(block, has_return))
                        }
                        ElseBranch::If(if_stmt) => Some(self.analyze_statement_with_return_check(
                            &Statement::If(*if_stmt.clone()),
                            has_return,
                        )),
                    }
                } else {
                    None
                };
                let mut children = vec![self.analyze_expression(&if_stmt.condition), then_node];
                if let Some(node) = else_node {
                    children.push(node);
                }
                AnnotatedNode {
                    node_type: "IfStatement".to_string(),
                    children,
                    ..Default::default()
                }
            }
            _ => self.analyze_statement(stmt),
        }
    }

    fn analyze_struct_declaration(&mut self, struct_decl: &StructDeclaration) -> AnnotatedNode {
        let name = &struct_decl.name.name;
        let mut fields = std::collections::HashMap::new();
        let mut field_nodes = vec![];

        for field in &struct_decl.fields {
            if fields.contains_key(&field.name.name) {
                self.errors.push(SemanticError::RedeclaredField(
                    name.clone(),
                    field.name.name.clone(),
                    field.name.line,
                    field.name.column,
                ));
            }
            fields.insert(field.name.name.clone(), field.field_type.clone());
            field_nodes.push(AnnotatedNode {
                node_type: "FieldDeclaration".to_string(),
                value: field.name.name.clone(),
                inferred_type: field.field_type.to_string(),
                start_line: field.name.line as u32,
                start_column: field.name.column as u32,
                ..Default::default()
            });
        }

        let symbol = Symbol::Struct {
            name: name.clone(),
            fields,
            line: struct_decl.name.line,
            column: struct_decl.name.column,
        };
        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredStruct(
                name.clone(),
                struct_decl.name.line,
                struct_decl.name.column,
            ));
        }

        AnnotatedNode {
            node_type: "StructDeclaration".to_string(),
            value: name.clone(),
            children: field_nodes,
            start_line: struct_decl.name.line as u32,
            start_column: struct_decl.name.column as u32,
            ..Default::default()
        }
    }

    fn analyze_statement(&mut self, statement: &Statement) -> AnnotatedNode {
        match statement {
            Statement::Expression(expr) => self.analyze_expression(expr),
            Statement::Block(block) => self.analyze_block(block),
            Statement::If(if_stmt) => {
                let cond_node = self.analyze_expression(&if_stmt.condition);
                let then_node = self.analyze_block(&if_stmt.then_block);
                let else_node = if_stmt.else_block.as_ref().map(|branch| match branch {
                    ElseBranch::Block(block) => self.analyze_statement(block),
                    ElseBranch::If(if_stmt) => {
                        self.analyze_statement(&Statement::If((**if_stmt).clone()))
                    }
                });
                let mut children = vec![cond_node, then_node];
                if let Some(node) = else_node {
                    children.push(node);
                }
                AnnotatedNode {
                    node_type: "IfStatement".to_string(),
                    children,
                    ..Default::default()
                }
            }
            Statement::While(while_stmt) => {
                let cond_node = self.analyze_expression(&while_stmt.condition);
                let body_node = self.analyze_block(&while_stmt.body);
                AnnotatedNode {
                    node_type: "WhileStatement".to_string(),
                    children: vec![cond_node, body_node],
                    ..Default::default()
                }
            }
            Statement::Return(return_stmt) => self.analyze_return_statement(return_stmt),
            Statement::For(for_stmt) => {
                self.symbol_table.enter_scope("for_loop".to_string());
                let var_name = &for_stmt.variable.name;
                let symbol = Symbol::Variable {
                    name: var_name.clone(),
                    type_: Type::Int, // Assuming loop variable is an integer
                    defined: true,
                    line: for_stmt.variable.line,
                    column: for_stmt.variable.column,
                    value: None,
                };
                self.symbol_table.insert(var_name.clone(), symbol);
                let iterable_node = self.analyze_expression(&for_stmt.iterable);
                let body_node = self.analyze_block(&for_stmt.body);
                self.symbol_table.leave_scope();
                AnnotatedNode {
                    node_type: "ForStatement".to_string(),
                    children: vec![
                        self.identifier_to_annotated(&for_stmt.variable),
                        iterable_node,
                        body_node,
                    ],
                    ..Default::default()
                }
            }
            _ => AnnotatedNode::default(),
        }
    }

    fn analyze_return_statement(&mut self, return_stmt: &ReturnStatement) -> AnnotatedNode {
        let (line, column) = return_stmt.value.get_line_col();
        let value_node = self.analyze_expression(&return_stmt.value);
        if let Some((_fn_name, return_type)) = &self.current_function {
            let expr_type = Type::from_str(&value_node.inferred_type).unwrap_or(Type::Void);
            if expr_type != *return_type {
                self.errors.push(SemanticError::ReturnTypeMismatch(
                    return_type.to_string(),
                    expr_type.to_string(),
                    line,
                    column,
                ));
            }
        } else {
            self.errors
                .push(SemanticError::ReturnOutsideFunction(line, column));
        }
        AnnotatedNode {
            node_type: "ReturnStatement".to_string(),
            children: vec![value_node],
            ..Default::default()
        }
    }

    fn analyze_block(&mut self, block: &Block) -> AnnotatedNode {
        self.symbol_table.enter_scope("block".to_string());
        let children = block
            .statements
            .iter()
            .map(|d| self.analyze_declaration(d))
            .collect();
        self.symbol_table.leave_scope();
        AnnotatedNode {
            node_type: "Block".to_string(),
            children,
            ..Default::default()
        }
    }

    fn analyze_expression(&mut self, expression: &Expression) -> AnnotatedNode {
        match expression {
            Expression::Identifier(id) => {
                let type_ = self.symbol_table.lookup(&id.name).map_or(Type::Void, |s| s.get_type());
                if self.symbol_table.lookup(&id.name).is_none() {
                    self.errors.push(SemanticError::UndeclaredVariable(
                        id.name.clone(),
                        id.line,
                        id.column,
                    ));
                }
                let mut node = self.identifier_to_annotated(id);
                node.inferred_type = type_.to_string();
                node
            }
            Expression::Literal(lit) => match lit {
                Literal::Int(v) => AnnotatedNode {
                    node_type: "IntLiteral".to_string(),
                    value: v.to_string(),
                    inferred_type: "Int".to_string(),
                    ..Default::default()
                },
                Literal::Float(v) => AnnotatedNode {
                    node_type: "FloatLiteral".to_string(),
                    value: v.to_string(),
                    inferred_type: "Float".to_string(),
                    ..Default::default()
                },
                Literal::String(v) => AnnotatedNode {
                    node_type: "StringLiteral".to_string(),
                    value: v.clone(),
                    inferred_type: "String".to_string(),
                    ..Default::default()
                },
                Literal::Bool(v) => AnnotatedNode {
                    node_type: "BoolLiteral".to_string(),
                    value: v.to_string(),
                    inferred_type: "Bool".to_string(),
                    ..Default::default()
                },
            },
            Expression::Binary { left, op, right } => {
                let left_node = self.analyze_expression(left);
                let right_node = self.analyze_expression(right);
                let left_type = Type::from_str(&left_node.inferred_type).unwrap_or(Type::Void);
                let right_type = Type::from_str(&right_node.inferred_type).unwrap_or(Type::Void);

                if left_type != right_type {
                    self.errors.push(SemanticError::TypeMismatch(
                        left_type.to_string(),
                        right_type.to_string(),
                        0, // Add line/col info
                        0,
                    ));
                }

                AnnotatedNode {
                    node_type: "BinaryExpression".to_string(),
                    value: format!("{:?}", op),
                    children: vec![left_node, right_node],
                    inferred_type: left_type.to_string(), // Simplification
                    ..Default::default()
                }
            }
            Expression::Assignment { target, value } => {
                let symbol_info = self.symbol_table.lookup(&target.name).map(|s| (s.is_constant(), s.get_type()));
                let value_node = self.analyze_expression(value);
                let value_type = Type::from_str(&value_node.inferred_type).unwrap_or(Type::Void);

                if let Some((is_constant, target_type)) = symbol_info {
                    if is_constant {
                        self.errors.push(SemanticError::InvalidAssignment(
                            format!("Cannot assign to constant '{}'", target.name),
                            target.line,
                            target.column,
                        ));
                    } else if target_type != value_type {
                        self.errors.push(SemanticError::TypeMismatch(
                            target_type.to_string(),
                            value_type.to_string(),
                            target.line,
                            target.column,
                        ));
                    }
                } else {
                    self.errors.push(SemanticError::UndeclaredVariable(
                        target.name.clone(),
                        target.line,
                        target.column,
                    ));
                }

                AnnotatedNode {
                    node_type: "Assignment".to_string(),
                    children: vec![self.identifier_to_annotated(target), value_node],
                    inferred_type: "Void".to_string(),
                    ..Default::default()
                }
            }
            Expression::FunctionCall { function, arguments } => {
                let fn_identifier = match &**function {
                    Expression::Identifier(ident) => ident,
                    _ => {
                        let (line, col) = function.get_line_col();
                        self.errors.push(SemanticError::InvalidFunctionCallTarget(line, col));
                        return AnnotatedNode {
                            node_type: "Error".to_string(),
                            value: "Invalid function call target".to_string(),
                            ..Default::default()
                        };
                    }
                };

                let mut arg_nodes = vec![];
                for arg in arguments {
                    arg_nodes.push(self.analyze_expression(arg));
                }

                let return_type = self.symbol_table.lookup(&fn_identifier.name).map_or(Type::Void, |s| s.get_type());

                AnnotatedNode {
                    node_type: "FunctionCall".to_string(),
                    value: fn_identifier.name.clone(),
                    children: arg_nodes,
                    inferred_type: return_type.to_string(),
                    ..Default::default()
                }
            }
            Expression::Unary { op, expr } => {
                let expr_node = self.analyze_expression(expr);
                let expr_type = Type::from_str(&expr_node.inferred_type).unwrap_or(Type::Void);
                
                // Unary operations preserve the type of their operand
                AnnotatedNode {
                    node_type: "UnaryExpression".to_string(),
                    value: format!("{:?}", op),
                    children: vec![expr_node],
                    inferred_type: expr_type.to_string(),
                    ..Default::default()
                }
            }
            Expression::Grouped(expr) => {
                // Grouped expressions just preserve the type of the inner expression
                let inner = self.analyze_expression(expr);
                AnnotatedNode {
                    node_type: "GroupedExpression".to_string(),
                    children: vec![inner.clone()],
                    inferred_type: inner.inferred_type,
                    ..Default::default()
                }
            }
            _ => AnnotatedNode {
                node_type: "UnsupportedExpression".to_string(),
                value: format!("{:?}", expression),
                inferred_type: "Void".to_string(),
                ..Default::default()
            },
        }
    }

    fn get_type(&self, opt_type: &Option<Type>) -> Type {
        opt_type.clone().unwrap_or(Type::Void)
    }

    fn identifier_to_annotated(&self, id: &Identifier) -> AnnotatedNode {
        AnnotatedNode {
            node_type: "Identifier".to_string(),
            value: id.name.clone(),
            start_line: id.line as u32,
            start_column: id.column as u32,
            end_line: id.line as u32,
            end_column: (id.column + id.name.len()) as u32,
            ..Default::default()
        }
    }
}
