use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, BasicMetadataValueEnum, BasicValue};
use inkwell::types::{BasicTypeEnum, BasicMetadataTypeEnum, BasicType};
use inkwell::{AddressSpace, IntPredicate, FloatPredicate};
use std::collections::HashMap;

use crate::ast::*;

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
    fpm: PassManager<FunctionValue<'ctx>>,
    variables: HashMap<String, PointerValue<'ctx>>,
    variable_types: HashMap<String, BasicTypeEnum<'ctx>>,
    current_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("dream_compiler");
        let fpm = PassManager::create(&module);

        // PassManager will optimize functions
        fpm.initialize();

        let compiler = Compiler {
            context,
            builder,
            module,
            fpm,
            variables: HashMap::new(),
            variable_types: HashMap::new(),
            current_function: None,
        };

        // Declare external C library functions
        compiler.declare_external_functions();

        compiler
    }

    /// Declare external C standard library functions like printf and puts
    fn declare_external_functions(&self) {
        // Declare printf: i32 printf(i8*, ...)
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true); // true = variadic
        self.module.add_function("printf", printf_type, None);

        // Declare puts: i32 puts(i8*)
        let puts_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        self.module.add_function("puts", puts_type, None);
    }

    pub fn compile(&mut self, program: &Program) -> Result<String, String> {
        for declaration in &program.declarations {
            self.compile_declaration(declaration)?;
        }

        if self.module.verify().is_err() {
            return Err("Module verification failed".to_string());
        }

        Ok(self.module.print_to_string().to_string())
    }

    fn compile_declaration(&mut self, declaration: &Declaration) -> Result<(), String> {
        match declaration {
            Declaration::Function(func) => self.compile_function(func),
            Declaration::Variable(var) => self.compile_global_variable(var),
            Declaration::Constant(const_decl) => self.compile_global_constant(const_decl),
            Declaration::Struct(_) => Ok(()), // Structs are handled separately
            Declaration::Statement(_) => Err("Top-level statements not supported".to_string()),
        }
    }

    fn compile_function(&mut self, function: &Function) -> Result<(), String> {
        let param_types: Vec<BasicMetadataTypeEnum> = function
            .parameters
            .iter()
            .map(|p| {
                self.ast_type_to_llvm(&p.param_type)
                    .and_then(|opt| opt.ok_or_else(|| "Void parameter type".to_string()))
                    .map(|t| t.into())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let fn_type = match self.ast_type_to_llvm(&function.return_type)? {
            Some(t) => t.fn_type(&param_types, false),
            None => self.context.void_type().fn_type(&param_types, false),
        };

        let fn_val = self.module.add_function(&function.name.name, fn_type, None);
        self.current_function = Some(fn_val);

        let entry = self.context.append_basic_block(fn_val, "entry");
        self.builder.position_at_end(entry);

        self.variables.clear();
        self.variable_types.clear();

        for (i, param) in function.parameters.iter().enumerate() {
            let param_val = fn_val.get_nth_param(i as u32).unwrap();
            let param_type = param_val.get_type();
            let alloca = self.create_entry_block_alloca(&param.name.name, param_type);
            self.builder.build_store(alloca, param_val).unwrap();
            self.variables.insert(param.name.name.clone(), alloca);
            self.variable_types.insert(param.name.name.clone(), param_type);
        }

        self.compile_block(&function.body)?;

        if function.return_type == Type::Void {
            if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                self.builder.build_return(None).unwrap();
            }
        }

        if fn_val.verify(true) {
            self.fpm.run_on(&fn_val);
            Ok(())
        } else {
            unsafe {
                fn_val.delete();
            }
            Err(format!("Invalid function: {}", function.name.name))
        }
    }

    fn compile_block(&mut self, block: &Block) -> Result<(), String> {
        for declaration in &block.statements {
            self.compile_block_declaration(declaration)?;
        }
        Ok(())
    }

    fn compile_block_declaration(&mut self, declaration: &Declaration) -> Result<(), String> {
        match declaration {
            Declaration::Variable(var) => {
                let value = self.compile_expression(&var.value)?;
                let var_type = value.get_type();
                let alloca = self.create_entry_block_alloca(&var.identifier.name, var_type);
                self.builder.build_store(alloca, value).unwrap();
                self.variables.insert(var.identifier.name.clone(), alloca);
                self.variable_types.insert(var.identifier.name.clone(), var_type);
                Ok(())
            }
            Declaration::Constant(const_decl) => {
                let value = self.compile_expression(&const_decl.value)?;
                let var_type = value.get_type();
                let alloca = self.create_entry_block_alloca(&const_decl.identifier.name, var_type);
                self.builder.build_store(alloca, value).unwrap();
                self.variables.insert(const_decl.identifier.name.clone(), alloca);
                self.variable_types.insert(const_decl.identifier.name.clone(), var_type);
                Ok(())
            }
            Declaration::Statement(stmt) => self.compile_statement(stmt),
            Declaration::Function(_) => Err("Nested functions not supported".to_string()),
            Declaration::Struct(_) => Ok(()),
        }
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                Ok(())
            }
            Statement::Return(ret) => {
                let value = self.compile_expression(&ret.value)?;
                self.builder.build_return(Some(&value)).unwrap();
                Ok(())
            }
            Statement::If(if_stmt) => self.compile_if(if_stmt),
            Statement::While(while_stmt) => self.compile_while(while_stmt),
            Statement::For(for_stmt) => self.compile_for(for_stmt),
            Statement::DoUntil(do_until) => self.compile_do_until(do_until),
            Statement::Block(block) => self.compile_block(block),
        }
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<BasicValueEnum<'ctx>, String> {
        match expression {
            Expression::Literal(lit) => self.compile_literal(lit),
            Expression::Identifier(ident) => {
                let ptr = self.variables.get(&ident.name)
                    .ok_or_else(|| format!("Undefined variable: {}", ident.name))?;
                let var_type = self.variable_types.get(&ident.name)
                    .ok_or_else(|| format!("Variable type not found: {}", ident.name))?;
                Ok(self.builder.build_load(*var_type, *ptr, &ident.name).unwrap())
            }
            Expression::Binary { left, op, right } => self.compile_binary(left, op, right),
            Expression::Unary { op, expr } => self.compile_unary(op, expr),
            Expression::Assignment { target, value } => {
                let val = self.compile_expression(value)?;
                let ptr = self.variables.get(&target.name)
                    .ok_or_else(|| format!("Undefined variable: {}", target.name))?;
                self.builder.build_store(*ptr, val).unwrap();
                Ok(val)
            }
            Expression::FunctionCall { function, arguments } => self.compile_function_call(function, arguments),
            Expression::Grouped(expr) => self.compile_expression(expr),
            _ => Err(format!("Unsupported expression type: {:?}", expression)),
        }
    }

    fn compile_literal(&self, literal: &Literal) -> Result<BasicValueEnum<'ctx>, String> {
        match literal {
            Literal::Int(val) => Ok(self.context.i64_type().const_int(*val as u64, true).into()),
            Literal::Float(val) => Ok(self.context.f64_type().const_float(*val).into()),
            Literal::Bool(val) => Ok(self.context.bool_type().const_int(*val as u64, false).into()),
            Literal::String(val) => {
                let global_str = self.builder.build_global_string_ptr(val, "str").unwrap();
                Ok(global_str.as_basic_value_enum())
            }
        }
    }

    fn compile_binary(&mut self, left: &Expression, op: &BinaryOp, right: &Expression) -> Result<BasicValueEnum<'ctx>, String> {
        let lhs = self.compile_expression(left)?;
        let rhs = self.compile_expression(right)?;

        match (lhs, rhs) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                let result = match op {
                    BinaryOp::Plus => self.builder.build_int_add(l, r, "tmpadd").unwrap(),
                    BinaryOp::Minus => self.builder.build_int_sub(l, r, "tmpsub").unwrap(),
                    BinaryOp::Asterisk => self.builder.build_int_mul(l, r, "tmpmul").unwrap(),
                    BinaryOp::Slash => self.builder.build_int_signed_div(l, r, "tmpdiv").unwrap(),
                    BinaryOp::Greater => return Ok(self.builder.build_int_compare(IntPredicate::SGT, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::Less => return Ok(self.builder.build_int_compare(IntPredicate::SLT, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::GreaterEqual => return Ok(self.builder.build_int_compare(IntPredicate::SGE, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::LessEqual => return Ok(self.builder.build_int_compare(IntPredicate::SLE, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::DoubleEqual => return Ok(self.builder.build_int_compare(IntPredicate::EQ, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::NotEqual => return Ok(self.builder.build_int_compare(IntPredicate::NE, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::DoubleAmpersand => self.builder.build_and(l, r, "tmpand").unwrap(),
                    BinaryOp::DoubleBar => self.builder.build_or(l, r, "tmpor").unwrap(),
                    _ => return Err(format!("Unsupported binary operation: {:?}", op)),
                };
                Ok(result.into())
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                let result = match op {
                    BinaryOp::Plus => self.builder.build_float_add(l, r, "tmpadd").unwrap(),
                    BinaryOp::Minus => self.builder.build_float_sub(l, r, "tmpsub").unwrap(),
                    BinaryOp::Asterisk => self.builder.build_float_mul(l, r, "tmpmul").unwrap(),
                    BinaryOp::Slash => self.builder.build_float_div(l, r, "tmpdiv").unwrap(),
                    BinaryOp::Greater => return Ok(self.builder.build_float_compare(FloatPredicate::OGT, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::Less => return Ok(self.builder.build_float_compare(FloatPredicate::OLT, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::GreaterEqual => return Ok(self.builder.build_float_compare(FloatPredicate::OGE, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::LessEqual => return Ok(self.builder.build_float_compare(FloatPredicate::OLE, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::DoubleEqual => return Ok(self.builder.build_float_compare(FloatPredicate::OEQ, l, r, "tmpcmp").unwrap().into()),
                    BinaryOp::NotEqual => return Ok(self.builder.build_float_compare(FloatPredicate::ONE, l, r, "tmpcmp").unwrap().into()),
                    _ => return Err(format!("Unsupported binary operation: {:?}", op)),
                };
                Ok(result.into())
            }
            _ => Err("Type mismatch in binary operation".to_string()),
        }
    }

    fn compile_unary(&mut self, op: &UnaryOp, expr: &Expression) -> Result<BasicValueEnum<'ctx>, String> {
        let val = self.compile_expression(expr)?;
        match op {
            UnaryOp::Minus => match val {
                BasicValueEnum::IntValue(i) => Ok(self.builder.build_int_neg(i, "tmpneg").unwrap().into()),
                BasicValueEnum::FloatValue(f) => Ok(self.builder.build_float_neg(f, "tmpneg").unwrap().into()),
                _ => Err("Cannot negate non-numeric value".to_string()),
            },
            UnaryOp::Exclamation => match val {
                BasicValueEnum::IntValue(i) => Ok(self.builder.build_not(i, "tmpnot").unwrap().into()),
                _ => Err("Cannot negate non-boolean value".to_string()),
            },
        }
    }

    fn compile_if(&mut self, if_stmt: &IfStatement) -> Result<(), String> {
        let condition = self.compile_expression(&if_stmt.condition)?;
        let condition = match condition {
            BasicValueEnum::IntValue(i) => i,
            _ => return Err("Condition must be boolean".to_string()),
        };

        let func = self.current_function.ok_or("No current function")?;
        let then_bb = self.context.append_basic_block(func, "then");
        let else_bb = self.context.append_basic_block(func, "else");
        let merge_bb = self.context.append_basic_block(func, "ifcont");

        self.builder.build_conditional_branch(condition, then_bb, else_bb).unwrap();

        // Compile then branch
        self.builder.position_at_end(then_bb);
        self.compile_block(&if_stmt.then_block)?;
        let then_has_terminator = self.builder.get_insert_block().unwrap().get_terminator().is_some();
        if !then_has_terminator {
            self.builder.build_unconditional_branch(merge_bb).unwrap();
        }

        // Compile else branch
        self.builder.position_at_end(else_bb);
        if let Some(else_branch) = &if_stmt.else_block {
            match else_branch {
                ElseBranch::If(inner_if) => self.compile_if(inner_if)?,
                ElseBranch::Block(block) => self.compile_statement(block)?,
            }
        }
        let else_has_terminator = self.builder.get_insert_block().unwrap().get_terminator().is_some();
        if !else_has_terminator {
            self.builder.build_unconditional_branch(merge_bb).unwrap();
        }

        // Only position at merge block if it will be used
        if !then_has_terminator || !else_has_terminator {
            self.builder.position_at_end(merge_bb);
        } else {
            // Both branches have terminators, so merge block is unreachable
            // We still need to position somewhere valid
            unsafe {
                merge_bb.delete().ok();
            }
        }
        Ok(())
    }

    fn compile_while(&mut self, while_stmt: &WhileStatement) -> Result<(), String> {
        let func = self.current_function.ok_or("No current function")?;
        let cond_bb = self.context.append_basic_block(func, "whilecond");
        let body_bb = self.context.append_basic_block(func, "whilebody");
        let after_bb = self.context.append_basic_block(func, "afterwhile");

        self.builder.build_unconditional_branch(cond_bb).unwrap();
        self.builder.position_at_end(cond_bb);

        let condition = self.compile_expression(&while_stmt.condition)?;
        let condition = match condition {
            BasicValueEnum::IntValue(i) => i,
            _ => return Err("Condition must be boolean".to_string()),
        };

        self.builder.build_conditional_branch(condition, body_bb, after_bb).unwrap();

        self.builder.position_at_end(body_bb);
        self.compile_block(&while_stmt.body)?;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(cond_bb).unwrap();
        }

        self.builder.position_at_end(after_bb);
        Ok(())
    }

    fn compile_for(&mut self, _for_stmt: &ForStatement) -> Result<(), String> {
        Err("For loops not yet implemented".to_string())
    }

    fn compile_do_until(&mut self, do_until: &DoUntilStatement) -> Result<(), String> {
        let func = self.current_function.ok_or("No current function")?;
        let body_bb = self.context.append_basic_block(func, "doBody");
        let cond_bb = self.context.append_basic_block(func, "doCond");
        let after_bb = self.context.append_basic_block(func, "afterDo");

        self.builder.build_unconditional_branch(body_bb).unwrap();
        self.builder.position_at_end(body_bb);

        self.compile_block(&do_until.body)?;
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.builder.build_unconditional_branch(cond_bb).unwrap();
        }

        self.builder.position_at_end(cond_bb);
        let condition = self.compile_expression(&do_until.condition)?;
        let condition = match condition {
            BasicValueEnum::IntValue(i) => i,
            _ => return Err("Condition must be boolean".to_string()),
        };

        self.builder.build_conditional_branch(condition, after_bb, body_bb).unwrap();
        self.builder.position_at_end(after_bb);
        Ok(())
    }

    fn compile_function_call(&mut self, function: &Expression, arguments: &[Expression]) -> Result<BasicValueEnum<'ctx>, String> {
        let func_name = match function {
            Expression::Identifier(ident) => &ident.name,
            _ => return Err("Function call target must be an identifier".to_string()),
        };

        let func = self.module.get_function(func_name)
            .ok_or_else(|| format!("Undefined function: {}", func_name))?;

        let args: Vec<BasicMetadataValueEnum> = arguments
            .iter()
            .map(|arg| self.compile_expression(arg).map(|v| v.into()))
            .collect::<Result<Vec<_>, _>>()?;

        let call_site = self.builder.build_call(func, &args, "tmp").unwrap();
        call_site.try_as_basic_value().left().ok_or("Function call returned void".to_string())
    }

    fn compile_global_variable(&mut self, var: &VariableDeclaration) -> Result<(), String> {
        let value = self.compile_expression(&var.value)?;
        let global = self.module.add_global(value.get_type(), Some(AddressSpace::default()), &var.identifier.name);
        
        match value {
            BasicValueEnum::IntValue(i) => global.set_initializer(&i),
            BasicValueEnum::FloatValue(f) => global.set_initializer(&f),
            _ => return Err("Global variables must be initialized with constants".to_string()),
        }
        
        Ok(())
    }

    fn compile_global_constant(&mut self, const_decl: &ConstantDeclaration) -> Result<(), String> {
        let value = self.compile_expression(&const_decl.value)?;
        let global = self.module.add_global(value.get_type(), Some(AddressSpace::default()), &const_decl.identifier.name);
        global.set_constant(true);
        
        match value {
            BasicValueEnum::IntValue(i) => global.set_initializer(&i),
            BasicValueEnum::FloatValue(f) => global.set_initializer(&f),
            _ => return Err("Global constants must be initialized with constants".to_string()),
        }
        
        Ok(())
    }

    fn create_entry_block_alloca(&self, name: &str, ty: impl inkwell::types::BasicType<'ctx>) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self.current_function.unwrap().get_first_basic_block().unwrap();
        
        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }
        
        builder.build_alloca(ty, name).unwrap()
    }

    fn ast_type_to_llvm(&self, ast_type: &Type) -> Result<Option<BasicTypeEnum<'ctx>>, String> {
        match ast_type {
            Type::Int => Ok(Some(self.context.i64_type().into())),
            Type::Float => Ok(Some(self.context.f64_type().into())),
            Type::Bool => Ok(Some(self.context.bool_type().into())),
            Type::String => Ok(Some(self.context.ptr_type(AddressSpace::default()).into())),
            Type::Void => Ok(None),
        }
    }
}

pub fn compile_to_llvm_ir(program: &Program) -> Result<String, String> {
    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    compiler.compile(program)
}
