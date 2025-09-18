use crate::ast::*;
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
}

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) {
        for declaration in &program.declarations {
            self.analyze_declaration(declaration);
        }
    }

    fn analyze_declaration(&mut self, declaration: &Declaration) {
        match declaration {
            Declaration::Variable(var_decl) => self.analyze_variable_declaration(var_decl),
            Declaration::Function(func_decl) => self.analyze_function_declaration(func_decl),
            Declaration::Struct(struct_decl) => self.analyze_struct_declaration(struct_decl),
            Declaration::Constant(const_decl) => self.analyze_constant_declaration(const_decl),
            Declaration::Statement(stmt) => self.analyze_statement(stmt),
        }
    }

    fn analyze_variable_declaration(&mut self, var_decl: &VariableDeclaration) {
        let name = &var_decl.identifier.name;
        let declared_type = self.get_type(&var_decl.var_type);
        let value_type = self.analyze_expression(&var_decl.value);

        if declared_type != Type::Void && declared_type != value_type {
            self.errors.push(SemanticError::TypeMismatch(declared_type.to_string(), value_type.to_string(), var_decl.identifier.line, var_decl.identifier.column));
        }

        let symbol = Symbol::Variable {
            name: name.clone(),
            type_: value_type,
            defined: true,
            line: var_decl.identifier.line,
            column: var_decl.identifier.column,
        };
        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(name.clone(), var_decl.identifier.line, var_decl.identifier.column));
        }
    }

    fn analyze_constant_declaration(&mut self, const_decl: &ConstantDeclaration) {
        let name = &const_decl.identifier.name;
        let declared_type = self.get_type(&const_decl.const_type);
        let value_type = self.analyze_expression(&const_decl.value);

        if declared_type != Type::Void && declared_type != value_type {
            self.errors.push(SemanticError::TypeMismatch(declared_type.to_string(), value_type.to_string(), const_decl.identifier.line, const_decl.identifier.column));
        }

        let symbol = Symbol::Variable {
            name: name.clone(),
            type_: value_type,
            defined: true,
            line: const_decl.identifier.line,
            column: const_decl.identifier.column,
        };
        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(name.clone(), const_decl.identifier.line, const_decl.identifier.column));
        }
    }

    fn analyze_function_declaration(&mut self, func_decl: &Function) {
        let name = &func_decl.name.name;
        let parameters: Vec<Type> = func_decl.parameters.iter().map(|p| p.param_type.clone()).collect();
        let symbol = Symbol::Function {
            name: name.clone(),
            parameters,
            return_type: func_decl.return_type.clone(),
            line: func_decl.name.line,
            column: func_decl.name.column,
        };

        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(name.clone(), func_decl.name.line, func_decl.name.column));
        }

        self.symbol_table.enter_scope();
        for param in &func_decl.parameters {
            let param_name = &param.name.name;
            let param_symbol = Symbol::Variable {
                name: param_name.clone(),
                type_: param.param_type.clone(),
                defined: true,
                line: param.name.line,
                column: param.name.column,
            };
            if !self.symbol_table.insert(param_name.clone(), param_symbol) {
                self.errors.push(SemanticError::RedeclaredVariable(param_name.clone(), param.name.line, param.name.column));
            }
        }
        self.analyze_block(&func_decl.body);
        self.symbol_table.leave_scope();
    }

    fn analyze_struct_declaration(&mut self, struct_decl: &StructDeclaration) {
        let name = &struct_decl.name.name;
        let mut fields = std::collections::HashMap::new();
        for field in &struct_decl.fields {
            if fields.contains_key(&field.name.name) {
                self.errors.push(SemanticError::RedeclaredField(name.clone(), field.name.name.clone(), field.name.line, field.name.column));
            }
            fields.insert(field.name.name.clone(), field.field_type.clone());
        }

        let symbol = Symbol::Struct {
            name: name.clone(),
            fields,
            line: struct_decl.name.line,
            column: struct_decl.name.column,
        };
        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredStruct(name.clone(), struct_decl.name.line, struct_decl.name.column));
        }
    }

    fn analyze_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expr) => {
                self.analyze_expression(expr);
            }
            Statement::Block(block) => {
                self.symbol_table.enter_scope();
                self.analyze_block(block);
                self.symbol_table.leave_scope();
            }
            Statement::If(if_stmt) => {
                self.analyze_expression(&if_stmt.condition);
                self.analyze_block(&if_stmt.then_block);
                if let Some(else_branch) = &if_stmt.else_block {
                    match else_branch {
                        ElseBranch::Block(block) => self.analyze_statement(block),
                        ElseBranch::If(if_stmt) => self.analyze_statement(&Statement::If((**if_stmt).clone())),
                    }
                }
            }
            Statement::While(while_stmt) => {
                self.analyze_expression(&while_stmt.condition);
                self.analyze_block(&while_stmt.body);
            }
            Statement::For(for_stmt) => {
                self.symbol_table.enter_scope();
                let var_name = &for_stmt.variable.name;
                let symbol = Symbol::Variable {
                    name: var_name.clone(),
                    type_: Type::Int, // Assuming loop variable is an integer
                    defined: true,
                    line: for_stmt.variable.line,
                    column: for_stmt.variable.column,
                };
                self.symbol_table.insert(var_name.clone(), symbol);
                self.analyze_expression(&for_stmt.iterable);
                self.analyze_block(&for_stmt.body);
                self.symbol_table.leave_scope();
            }
            _ => {}
        }
    }

    fn analyze_block(&mut self, block: &Block) {
        for declaration in &block.statements {
            self.analyze_declaration(declaration);
        }
    }

    fn analyze_expression(&mut self, expression: &Expression) -> Type {
        match expression {
            Expression::Identifier(id) => {
                if let Some(symbol) = self.symbol_table.lookup(&id.name) {
                    match symbol {
                        Symbol::Variable { type_, .. } => return type_.clone(),
                        _ => return Type::Void, // Or some other appropriate type for non-variables
                    }
                }
                self.errors.push(SemanticError::UndeclaredVariable(id.name.clone(), id.line, id.column));
                Type::Void
            }
            Expression::Literal(lit) => match lit {
                Literal::Int(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::String(_) => Type::String,
                Literal::Bool(_) => Type::Bool,
            },
            Expression::Binary { left, op: _, right } => {
                let left_type = self.analyze_expression(left);
                let right_type = self.analyze_expression(right);
                if left_type != right_type {
                    // This is a simplification. In a real compiler, you'd have more complex type compatibility rules.
                    self.errors.push(SemanticError::TypeMismatch(left_type.to_string(), right_type.to_string(), 0, 0)); // Add line/col info
                }
                left_type // For simplicity, returning left_type. Should be based on operator.
            }
            Expression::Assignment { target, value } => {
                if let Some(symbol) = self.symbol_table.lookup(&target.name) {
                    let target_type = match symbol {
                        Symbol::Variable { type_, .. } => type_.clone(),
                        _ => Type::Void,
                    };
                    let value_type = self.analyze_expression(value);
                    if target_type != value_type {
                        self.errors.push(SemanticError::TypeMismatch(target_type.to_string(), value_type.to_string(), target.line, target.column));
                    }
                } else {
                    self.errors.push(SemanticError::UndeclaredVariable(target.name.clone(), target.line, target.column));
                }
                Type::Void
            }
            Expression::StructInstantiation { name, fields } => {
                if let Some(Symbol::Struct { fields: struct_fields, .. }) = self.symbol_table.lookup(&name.name).cloned() {
                    for (field_name, field_expr) in fields {
                        if let Some(field_type) = struct_fields.get(&field_name.name) {
                            let expr_type = self.analyze_expression(field_expr);
                            if *field_type != expr_type {
                                self.errors.push(SemanticError::TypeMismatch(field_type.to_string(), expr_type.to_string(), field_name.line, field_name.column));
                            }
                        } else {
                            self.errors.push(SemanticError::FieldNotFound(name.name.clone(), field_name.name.clone(), field_name.line, field_name.column));
                        }
                    }
                    return Type::Void; // Should be a struct type
                }
                self.errors.push(SemanticError::UndefinedStruct(name.name.clone(), name.line, name.column));
                Type::Void
            }
            Expression::MemberAccess { object, property } => {
                let object_type = self.analyze_expression(object);
                // This is a simplification. You would need to get the struct definition and check the field.
                Type::Void
            }
            _ => Type::Void,
        }
    }

    fn get_type(&self, opt_type: &Option<Type>) -> Type {
        opt_type.clone().unwrap_or(Type::Void)
    }
}

impl Type {
    pub fn to_string(&self) -> String {
        match self {
            Type::Int => "Int".to_string(),
            Type::Float => "Float".to_string(),
            Type::String => "String".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::Void => "Void".to_string(),
        }
    }
}