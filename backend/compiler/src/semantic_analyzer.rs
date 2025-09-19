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

    pub fn analyze(&mut self, program: &Program) {
        for declaration in &program.declarations {
            self.analyze_declaration(declaration);
        }
        self.check_for_main_function();
    }

    fn check_for_main_function(&mut self) {
        match self.symbol_table.lookup("main") {
            Some(symbol) => {
                // A symbol named 'main' was found, now check if it's a valid function
                if let Symbol::Function {
                    parameters,
                    return_type,
                    line,
                    column,
                    ..
                } = symbol
                {
                    let mut signature_errors = Vec::new();

                    // 1. Check if the function takes any parameters
                    if !parameters.is_empty() {
                        signature_errors.push(format!(
                            "expected 0 parameters but found {}",
                            parameters.len()
                        ));
                    }

                    // 2. Check if the function returns 'Int'
                    if *return_type != Type::Int {
                        signature_errors.push(format!(
                            "expected a 'Int' return type but found '{}'",
                            return_type.to_string()
                        ));
                    }

                    // 3. If there are any signature errors, report them
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
                    // A symbol 'main' exists, but it is not a function (e.g., a variable)
                    self.errors.push(SemanticError::MissingMainFunction);
                }
            }
            None => {
                // No symbol named 'main' was found in the global scope
                self.errors.push(SemanticError::MissingMainFunction);
            }
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
            self.errors.push(SemanticError::TypeMismatch(
                declared_type.to_string(),
                value_type.to_string(),
                var_decl.identifier.line,
                var_decl.identifier.column,
            ));
        }

        let symbol = Symbol::Variable {
            name: name.clone(),
            type_: value_type,
            defined: true,
            line: var_decl.identifier.line,
            column: var_decl.identifier.column,
        };
        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(
                name.clone(),
                var_decl.identifier.line,
                var_decl.identifier.column,
            ));
        }
    }

    fn analyze_constant_declaration(&mut self, const_decl: &ConstantDeclaration) {
        let name = &const_decl.identifier.name;
        let declared_type = self.get_type(&const_decl.const_type);
        let value_type = self.analyze_expression(&const_decl.value);

        if declared_type != Type::Void && declared_type != value_type {
            self.errors.push(SemanticError::TypeMismatch(
                declared_type.to_string(),
                value_type.to_string(),
                const_decl.identifier.line,
                const_decl.identifier.column,
            ));
        }

        let symbol = Symbol::Constant {
            // Changed to Constant variant
            name: name.clone(),
            type_: value_type,
            line: const_decl.identifier.line,
            column: const_decl.identifier.column,
        };

        if !self.symbol_table.insert(name.clone(), symbol) {
            self.errors.push(SemanticError::RedeclaredVariable(
                name.clone(),
                const_decl.identifier.line,
                const_decl.identifier.column,
            ));
        }
    }

    fn analyze_function_declaration(&mut self, func_decl: &Function) {
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

        // Set current function context
        let previous_function = self.current_function.take();
        self.current_function = Some((name.clone(), return_type.clone()));

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
                self.errors.push(SemanticError::RedeclaredVariable(
                    param_name.clone(),
                    param.name.line,
                    param.name.column,
                ));
            }
        }

        // Analyze function body and track returns
        let mut has_return = false;
        self.analyze_block_with_return_check(&func_decl.body, &mut has_return);

        // Check if non-void function has at least one return statement
        // Note: This is a simple check and doesn't account for control flow
        if return_type != Type::Void && !has_return {
            // We would need to implement more sophisticated control flow analysis
            // to properly check if all paths return a value
            // For now, we'll just add a placeholder check
            self.errors.push(SemanticError::MissingReturnStatement(
                name.clone(),
                func_decl.name.line,
                func_decl.name.column,
            ));
        }

        self.symbol_table.leave_scope();

        // Restore previous function context
        self.current_function = previous_function;
    }

    fn analyze_block_with_return_check(&mut self, block: &Block, has_return: &mut bool) {
        for decl in &block.statements {
            if let Declaration::Statement(stmt) = decl {
                self.analyze_statement_with_return_check(stmt, has_return);
            } else {
                self.analyze_declaration(decl);
            }
        }
    }

    fn analyze_statement_with_return_check(&mut self, stmt: &Statement, has_return: &mut bool) {
        match stmt {
            Statement::Return(_) => *has_return = true,
            Statement::Block(block) => self.analyze_block_with_return_check(block, has_return),
            Statement::If(if_stmt) => {
                self.analyze_statement_with_return_check(
                    &Statement::Block(if_stmt.then_block.clone()),
                    has_return,
                );
                if let Some(else_branch) = &if_stmt.else_block {
                    match else_branch {
                        ElseBranch::Block(block) => {
                            self.analyze_statement_with_return_check(block, has_return)
                        }
                        ElseBranch::If(if_stmt) => self.analyze_statement_with_return_check(
                            &Statement::If(*if_stmt.clone()),
                            has_return,
                        ),
                    }
                }
            }
            // Handle other statement types...
            _ => self.analyze_statement(stmt),
        }
    }

    fn analyze_struct_declaration(&mut self, struct_decl: &StructDeclaration) {
        let name = &struct_decl.name.name;
        let mut fields = std::collections::HashMap::new();
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
                        ElseBranch::If(if_stmt) => {
                            self.analyze_statement(&Statement::If((**if_stmt).clone()))
                        }
                    }
                }
            }
            Statement::While(while_stmt) => {
                self.analyze_expression(&while_stmt.condition);
                self.analyze_block(&while_stmt.body);
            }
            Statement::Return(return_stmt) => {
                let (line, column) = return_stmt.value.get_line_col();
                // Check if we're inside a function
                if let Some((fn_name, return_type)) = &self.current_function {
                    let return_type = return_type.clone();
                    let expr_type = self.analyze_expression(&return_stmt.value);
                    if expr_type != return_type {
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
                self.errors.push(SemanticError::UndeclaredVariable(
                    id.name.clone(),
                    id.line,
                    id.column,
                ));
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
                    self.errors.push(SemanticError::TypeMismatch(
                        left_type.to_string(),
                        right_type.to_string(),
                        0,
                        0,
                    )); // Add line/col info
                }
                left_type // For simplicity, returning left_type. Should be based on operator.
            }
            Expression::Assignment { target, value } => {
                // First, get the symbol info without holding the reference
                let symbol_info =
                    self.symbol_table
                        .lookup(&target.name)
                        .map(|symbol| match symbol {
                            Symbol::Constant { .. } => (true, symbol.get_type()),
                            Symbol::Variable { .. } => (false, symbol.get_type()),
                            _ => (false, Type::Void),
                        });

                match symbol_info {
                    Some((is_constant, target_type)) => {
                        if is_constant {
                            self.errors.push(SemanticError::InvalidAssignment(
                                format!("Cannot assign to constant '{}'", target.name),
                                target.line,
                                target.column,
                            ));
                        } else {
                            let value_type = self.analyze_expression(value);
                            if target_type != value_type {
                                self.errors.push(SemanticError::TypeMismatch(
                                    target_type.to_string(),
                                    value_type.to_string(),
                                    target.line,
                                    target.column,
                                ));
                            }
                        }
                    }
                    None => {
                        self.errors.push(SemanticError::UndeclaredVariable(
                            target.name.clone(),
                            target.line,
                            target.column,
                        ));
                    }
                }
                Type::Void
            }
            Expression::MemberAccess { object, property } => {
                let object_type = self.analyze_expression(object);
                // This is a simplification. You would need to get the struct definition and check the field.
                Type::Void
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                let fn_identifier = match &**function {
                    Expression::Identifier(ident) => ident,
                    _ => {
                        self.errors.push(SemanticError::InvalidFunctionCallTarget(
                            function.get_line_col().0,
                            function.get_line_col().1,
                        ));
                        return Type::Void;
                    }
                };

                if let Some(symbol) = self.symbol_table.lookup(&fn_identifier.name) {
                    match symbol {
                        Symbol::Function {
                            name: fn_name_sym,
                            parameters: parameters_sym,
                            return_type: return_type_sym,
                            ..
                        } => {
                            let fn_name = fn_name_sym.clone();
                            let parameters = parameters_sym.clone();
                            let return_type = return_type_sym.clone();

                            // Check argument count
                            if arguments.len() != parameters.len() {
                                self.errors.push(SemanticError::ArgumentCountMismatch(
                                    fn_name.to_string(),
                                    parameters.len(),
                                    arguments.len(),
                                    fn_identifier.line,
                                    fn_identifier.column,
                                ));
                                return return_type.clone();
                            }

                            // Check argument types
                            let param_types = parameters.clone();
                            for (i, (arg, param_type)) in
                                arguments.iter().zip(param_types.iter()).enumerate()
                            {
                                let arg_type = self.analyze_expression(arg);
                                if arg_type != *param_type {
                                    self.errors.push(SemanticError::ArgumentTypeMismatch(
                                        fn_name.clone(),
                                        i,
                                        param_type.to_string(),
                                        arg_type.to_string(),
                                        fn_identifier.line,
                                        fn_identifier.column,
                                    ));
                                }
                            }

                            return_type.clone()
                        }
                        _ => {
                            self.errors.push(SemanticError::UndefinedFunction(
                                fn_identifier.name.clone(),
                                fn_identifier.line,
                                fn_identifier.column,
                            ));
                            Type::Void
                        }
                    }
                } else {
                    self.errors.push(SemanticError::UndefinedFunction(
                        fn_identifier.name.clone(),
                        fn_identifier.line,
                        fn_identifier.column,
                    ));
                    Type::Void
                }
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
