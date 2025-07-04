use inkwell::AddressSpace;
use inkwell::FloatPredicate;
use inkwell::IntPredicate;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue};
use rune_parser::parser::expr::Expr;
use rune_parser::parser::nodes::Nodes;
use rune_parser::parser::ops::{BinaryOp, UnaryOp};
use rune_parser::parser::types::Types;
use std::collections::HashMap;

use crate::errors::CodeGenError;

pub struct CodeGen<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
    function: Option<FunctionValue<'ctx>>,
    puts_fn: Option<FunctionValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            function: None,
            puts_fn: None,
        }
    }

    pub fn create_main_function(&mut self) {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);
        self.function = Some(function);
        self.declare_puts_function();
    }

    fn declare_puts_function(&mut self) {
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.ptr_type(AddressSpace::default());
        let puts_fn_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let puts_fn = self.module.add_function("puts", puts_fn_type, None);
        self.puts_fn = Some(puts_fn);
    }
}

// Core
impl<'ctx> CodeGen<'ctx> {
    pub fn compile_statements(&mut self, statements: &[Expr]) -> Result<(), CodeGenError> {
        if self.function.is_none() {
            self.create_main_function();
        }

        for statement in statements {
            self.compile_expression(statement)?;
        }

        // Return 0 from main
        let zero = self.context.i32_type().const_int(0, false);
        let built_return = self.builder.build_return(Some(&zero));

        if built_return.is_err() {
            return Err(CodeGenError::TypeMismatchCustom(
                "Return must be an integer".to_string(),
            ));
        }

        Ok(())
    }

    pub fn compile_expression(
        &mut self,
        expr: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match expr {
            Expr::Literal(Nodes::Identifier(name)) => {
                if let Some((var_ptr, pointee_type)) = self.variables.get(name) {
                    let loaded_val = self
                        .builder
                        .build_load(*pointee_type, *var_ptr, name)
                        .unwrap();
                    Ok(loaded_val)
                } else {
                    Err(CodeGenError::UndefinedVariable(name.clone()))
                }
            }
            Expr::Literal(node) => self.compile_literal(node),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.compile_binary_op(left, operator, right),
            Expr::Unary { operator, operand } => self.compile_unary_op(operator, operand),
            Expr::Assignment { identifier, value } => self.compile_assignment(identifier, value),
            Expr::LetDeclaration {
                identifier,
                value,
                var_type,
            } => self.compile_let_declaration(identifier, value, var_type),
            Expr::IfElse {
                condition,
                then_branch,
                else_branch,
            } => self.compile_if_else(condition, then_branch, else_branch),
            Expr::Block(statements) => self.compile_block(statements),
            Expr::Print(expr) => self.compile_print(expr),
            Expr::MethodCall {
                target,
                method_name,
                arguments,
            } => todo!(),
        }
    }

    fn compile_literal(&self, node: &Nodes) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match node {
            Nodes::Integer(value) => {
                let int_val = self.context.i64_type().const_int(*value as u64, true);
                Ok(int_val.into())
            }
            Nodes::Float(value) => {
                let float_val = self.context.f64_type().const_float(*value);
                Ok(float_val.into())
            }
            Nodes::Boolean(value) => {
                let bool_val = self.context.bool_type().const_int(*value as u64, false);
                Ok(bool_val.into())
            }
            Nodes::String(value) => {
                let string_val = self.builder.build_global_string_ptr(value, "str");

                match string_val {
                    Ok(global_val) => Ok(global_val.as_pointer_value().into()),
                    Err(err) => Err(CodeGenError::StringError(err.to_string())),
                }
            }
            Nodes::Identifier(name) => Err(CodeGenError::InternalError(format!(
                "Unexpected identifier node {} in literal position",
                name
            ))),
        }
    }
}

// Operations
impl<'ctx> CodeGen<'ctx> {
    fn compile_binary_op(
        &mut self,
        left: &Expr,
        operator: &BinaryOp,
        right: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let left_val = self.compile_expression(left)?;
        let right_val = self.compile_expression(right)?;

        match (left_val, right_val) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                self.compile_int_binary_op(l, operator, r)
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                self.compile_float_binary_op(l, operator, r)
            }
            (BasicValueEnum::IntValue(l), BasicValueEnum::FloatValue(r)) => {
                let l_float = self
                    .builder
                    .build_signed_int_to_float(l, self.context.f64_type(), "int_to_float")
                    .unwrap();
                self.compile_float_binary_op(l_float, operator, r)
            }
            (BasicValueEnum::FloatValue(l), BasicValueEnum::IntValue(r)) => {
                let r_float = self
                    .builder
                    .build_signed_int_to_float(r, self.context.f64_type(), "int_to_float")
                    .unwrap();
                self.compile_float_binary_op(l, operator, r_float)
            }
            (BasicValueEnum::PointerValue(l), BasicValueEnum::PointerValue(r)) => {
                self.compile_ptr_binary_op(l, operator, r)
            }
            (BasicValueEnum::PointerValue(l), BasicValueEnum::IntValue(r)) => {
                let r_ptr = self
                    .builder
                    .build_int_to_ptr(
                        r,
                        self.context.ptr_type(AddressSpace::default()),
                        "int_to_ptr",
                    )
                    .unwrap();
                self.compile_ptr_binary_op(l, operator, r_ptr)
            }
            (BasicValueEnum::IntValue(l), BasicValueEnum::PointerValue(r)) => {
                let l_ptr = self
                    .builder
                    .build_int_to_ptr(
                        l,
                        self.context.ptr_type(AddressSpace::default()),
                        "int_to_ptr",
                    )
                    .unwrap();
                self.compile_ptr_binary_op(l_ptr, operator, r)
            }
            _ => Err(CodeGenError::InternalError(format!(
                "No binary operator for {:?} | {:?}",
                left_val.get_type(),
                right_val.get_type()
            ))),
        }
    }

    fn compile_ptr_binary_op(
        &self,
        left: PointerValue<'ctx>,
        operator: &BinaryOp,
        right: PointerValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match operator {
            BinaryOp::Add => {
                let result = self.builder.build_int_add(left, right, "add").unwrap();
                Ok(BasicValueEnum::PointerValue(result))
            }
            BinaryOp::Subtract => {
                let result = self.builder.build_int_sub(left, right, "sub").unwrap();
                Ok(BasicValueEnum::PointerValue(result))
            }
            BinaryOp::Multiply => {
                let result = self.builder.build_int_mul(left, right, "mul").unwrap();
                Ok(BasicValueEnum::PointerValue(result))
            }
            BinaryOp::Divide => {
                let result = self
                    .builder
                    .build_int_unsigned_div(left, right, "div")
                    .unwrap();
                Ok(BasicValueEnum::PointerValue(result))
            }
            BinaryOp::Modulo => {
                let result = self
                    .builder
                    .build_int_unsigned_rem(left, right, "rem")
                    .unwrap();
                Ok(BasicValueEnum::PointerValue(result))
            }
            BinaryOp::Greater => {
                let result = self
                    .builder
                    .build_int_compare(IntPredicate::UGT, left, right, "gt")
                    .unwrap();
                Ok(BasicValueEnum::IntValue(result))
            }
            _ => Err(CodeGenError::OperatorNotSupported(
                format!("{:?}", operator),
                format!("{:?} | {:?}", left.get_type(), right.get_type()),
            )),
        }
    }

    fn compile_int_binary_op(
        &self,
        left: IntValue<'ctx>,
        operator: &BinaryOp,
        right: IntValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let result = match operator {
            BinaryOp::Add => self.builder.build_int_add(left, right, "add").unwrap(),
            BinaryOp::Subtract => self.builder.build_int_sub(left, right, "sub").unwrap(),
            BinaryOp::Multiply => self.builder.build_int_mul(left, right, "mul").unwrap(),
            BinaryOp::Divide => self
                .builder
                .build_int_signed_div(left, right, "div")
                .unwrap(),
            BinaryOp::Modulo => self
                .builder
                .build_int_signed_rem(left, right, "rem")
                .unwrap(),
            BinaryOp::Equal => self
                .builder
                .build_int_compare(IntPredicate::EQ, left, right, "eq")
                .unwrap(),
            BinaryOp::NotEqual => self
                .builder
                .build_int_compare(IntPredicate::NE, left, right, "ne")
                .unwrap(),
            BinaryOp::Greater => self
                .builder
                .build_int_compare(IntPredicate::SGT, left, right, "gt")
                .unwrap(),
            BinaryOp::Less => self
                .builder
                .build_int_compare(IntPredicate::SLT, left, right, "lt")
                .unwrap(),
            BinaryOp::GreaterEqual => self
                .builder
                .build_int_compare(IntPredicate::SGE, left, right, "ge")
                .unwrap(),
            BinaryOp::LessEqual => self
                .builder
                .build_int_compare(IntPredicate::SLE, left, right, "le")
                .unwrap(),
            BinaryOp::And => self.builder.build_and(left, right, "and").unwrap(),
            BinaryOp::Or => self.builder.build_or(left, right, "or").unwrap(),
        };
        Ok(result.into())
    }

    fn compile_float_binary_op(
        &self,
        left: FloatValue<'ctx>,
        operator: &BinaryOp,
        right: FloatValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        match operator {
            BinaryOp::Add => {
                let result = self.builder.build_float_add(left, right, "fadd").unwrap();
                Ok(result.into())
            }
            BinaryOp::Subtract => {
                let result = self.builder.build_float_sub(left, right, "fsub").unwrap();
                Ok(result.into())
            }
            BinaryOp::Multiply => {
                let result = self.builder.build_float_mul(left, right, "fmul").unwrap();
                Ok(result.into())
            }
            BinaryOp::Divide => {
                let result = self.builder.build_float_div(left, right, "fdiv").unwrap();
                Ok(result.into())
            }
            BinaryOp::Modulo => {
                let result = self.builder.build_float_rem(left, right, "frem").unwrap();
                Ok(result.into())
            }
            BinaryOp::Equal => {
                let result = self
                    .builder
                    .build_float_compare(FloatPredicate::OEQ, left, right, "feq")
                    .unwrap();
                Ok(result.into())
            }
            BinaryOp::NotEqual => {
                let result = self
                    .builder
                    .build_float_compare(FloatPredicate::ONE, left, right, "fne")
                    .unwrap();
                Ok(result.into())
            }
            BinaryOp::Greater => {
                let result = self
                    .builder
                    .build_float_compare(FloatPredicate::OGT, left, right, "fgt")
                    .unwrap();
                Ok(result.into())
            }
            BinaryOp::Less => {
                let result = self
                    .builder
                    .build_float_compare(FloatPredicate::OLT, left, right, "flt")
                    .unwrap();
                Ok(result.into())
            }
            BinaryOp::GreaterEqual => {
                let result = self
                    .builder
                    .build_float_compare(FloatPredicate::OGE, left, right, "fge")
                    .unwrap();
                Ok(result.into())
            }
            BinaryOp::LessEqual => {
                let result = self
                    .builder
                    .build_float_compare(FloatPredicate::OLE, left, right, "fle")
                    .unwrap();
                Ok(result.into())
            }
            BinaryOp::And | BinaryOp::Or => Err(CodeGenError::InvalidOperation(
                "Logical operations not supported on floats".to_string(),
            )),
        }
    }

    fn compile_unary_op(
        &mut self,
        operator: &UnaryOp,
        operand: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let operand_val = self.compile_expression(operand)?;

        match operator {
            UnaryOp::Minus => match operand_val {
                BasicValueEnum::IntValue(int_val) => {
                    let result = self.builder.build_int_neg(int_val, "neg").unwrap();
                    Ok(result.into())
                }
                BasicValueEnum::FloatValue(float_val) => {
                    let result = self.builder.build_float_neg(float_val, "fneg").unwrap();
                    Ok(result.into())
                }
                _ => Err(CodeGenError::OperatorNotSupported(
                    "-".into(),
                    operand.to_string(),
                )),
            },
            UnaryOp::Not => match operand_val {
                BasicValueEnum::IntValue(int_val) => {
                    let result = self.builder.build_not(int_val, "not").unwrap();
                    Ok(result.into())
                }
                _ => Err(CodeGenError::OperatorNotSupported(
                    "!".into(),
                    operand.to_string(),
                )),
            },
        }
    }
}

// Assignments
impl<'ctx> CodeGen<'ctx> {
    fn compile_assignment(
        &mut self,
        identifier: &str,
        value: &Expr,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let val = self.compile_expression(value)?;

        if let Some((var_ptr, _)) = self.variables.get(identifier) {
            self.builder.build_store(*var_ptr, val).unwrap();
            Ok(val)
        } else {
            Err(CodeGenError::UndefinedVariable(identifier.to_string()))
        }
    }

    fn compile_let_declaration(
        &mut self,
        identifier: &str,
        value: &Expr,
        var_type: &Option<Types>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let val = self.compile_expression(value)?;

        // Use the specified type instead of inferring from value
        let llvm_type = match var_type {
            Some(Types::I32) => self.context.i32_type().into(),
            Some(Types::I64) => self.context.i64_type().into(),
            Some(Types::F32) => self.context.f32_type().into(),
            Some(Types::F64) => self.context.f64_type().into(),
            Some(Types::Bool) => self.context.bool_type().into(),
            Some(Types::String) => self.context.ptr_type(AddressSpace::default()).into(),
            None => self.context.i64_type().into(),
        };

        let alloca = self.builder.build_alloca(llvm_type, identifier).unwrap();

        let result = self.builder.build_store(alloca, val);

        if result.is_err() {
            return Err(CodeGenError::StoreError(identifier.to_string()));
        }

        self.variables
            .insert(identifier.to_string(), (alloca, llvm_type));

        Ok(val)
    }
}

// If-Else
impl<'ctx> CodeGen<'ctx> {
    fn compile_if_else(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: &Option<Box<Expr>>,
    ) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let function = self.function.ok_or(CodeGenError::NoFunction).unwrap();

        let condition_val = self.compile_expression(condition)?;

        let condition_bool = match condition_val {
            BasicValueEnum::IntValue(int_val) => {
                if int_val.get_type().get_bit_width() == 1 {
                    int_val
                } else {
                    let zero = int_val.get_type().const_zero();
                    self.builder
                        .build_int_compare(IntPredicate::NE, int_val, zero, "tobool")
                        .unwrap()
                }
            }
            _ => {
                return Err(CodeGenError::TypeMismatchCustom(
                    "Condition must be an integer".to_string(),
                ));
            }
        };

        let then_bb = self.context.append_basic_block(function, "then");
        let else_bb = self.context.append_basic_block(function, "else");
        let merge_bb = self.context.append_basic_block(function, "ifcont");

        let built_cond_branch =
            self.builder
                .build_conditional_branch(condition_bool, then_bb, else_bb);

        if built_cond_branch.is_err() {
            return Err(CodeGenError::TypeMismatchCustom(
                "Condition must be an integer".to_string(),
            ));
        }

        self.builder.position_at_end(then_bb);
        let then_val = self.compile_expression(then_branch)?;
        let built_unconditional_branch = self.builder.build_unconditional_branch(merge_bb);

        if built_unconditional_branch.is_err() {
            return Err(CodeGenError::TypeMismatchCustom(
                "Branch must be an integer".to_string(),
            ));
        }

        let then_bb_end = self.builder.get_insert_block().unwrap();

        self.builder.position_at_end(else_bb);
        let else_val = if let Some(else_expr) = else_branch {
            self.compile_expression(else_expr)?
        } else {
            self.context.i64_type().const_int(0, false).into()
        };

        let built_unconditional_branch = self.builder.build_unconditional_branch(merge_bb);

        if built_unconditional_branch.is_err() {
            return Err(CodeGenError::TypeMismatchCustom(
                "Branch must be an integer".to_string(),
            ));
        }

        let else_bb_end = self.builder.get_insert_block().unwrap();

        // merge block with phi node
        self.builder.position_at_end(merge_bb);

        // Only create phi if both branches have the same type
        if then_val.get_type() == else_val.get_type() {
            let phi = self
                .builder
                .build_phi(then_val.get_type(), "iftmp")
                .unwrap();
            phi.add_incoming(&[(&then_val, then_bb_end), (&else_val, else_bb_end)]);
            Ok(phi.as_basic_value())
        } else {
            Ok(then_val)
        }
    }
}

// Block
impl<'ctx> CodeGen<'ctx> {
    fn compile_block(&mut self, statements: &[Expr]) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let mut last_val = self.context.i64_type().const_int(0, false).into();

        for statement in statements {
            last_val = self.compile_expression(statement)?;
        }

        Ok(last_val)
    }
}

// Display
impl<'ctx> CodeGen<'ctx> {
    pub fn print_ir(&self) {
        self.module.print_to_stderr();
    }

    pub fn get_ir_string(&self) -> String {
        self.module.print_to_string().to_string()
    }
}

// Print
impl<'ctx> CodeGen<'ctx> {
    fn compile_print(&mut self, value: &Expr) -> Result<BasicValueEnum<'ctx>, CodeGenError> {
        let printed_val = self.compile_expression(value)?;

        let puts_fn = self.puts_fn.ok_or(CodeGenError::InternalError(
            "puts function not declared".to_string(),
        ))?;

        let printed_val_i8_ptr: BasicValueEnum<'ctx> = match printed_val {
            BasicValueEnum::PointerValue(ptr_val) => ptr_val.into(),
            BasicValueEnum::IntValue(_int_val) => {
                // If it's an integer, we need to convert it to a string.
                // This is a simplified approach, for a robust solution you'd
                // likely need a runtime function to convert integers to strings.
                // For now, let's assume we are printing string literals directly.
                // If you want to print integers, you'd need `sprintf` or similar.
                return Err(CodeGenError::TypeMismatchCustom(
                    "Printing integers directly not supported yet. Only strings.".to_string(),
                ));
            }
            _ => {
                return Err(CodeGenError::TypeMismatchCustom(
                    "Only strings can be printed directly for now.".to_string(),
                ));
            }
        };

        let call_result = self
            .builder
            .build_call(puts_fn, &[printed_val_i8_ptr.into()], "puts_call")
            .unwrap();

        Ok(call_result.try_as_basic_value().left().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune_parser::parser::Parser;

    #[test]
    fn test_simple_arithmetic() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "test");

        let mut parser = Parser::new("let x = 5 + 3".to_string()).unwrap();
        let statements = parser.parse().unwrap();

        codegen.compile_statements(&statements).unwrap();

        // Verify module is valid
        assert_ne!(codegen.module.to_string(), "");
        assert!(codegen.module.verify().is_ok());
    }

    #[test]
    fn test_variables() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "test");

        let mut parser = Parser::new("let x = 10; let y = x + 5".to_string()).unwrap();
        let statements = parser.parse().unwrap();

        codegen.compile_statements(&statements).unwrap();

        let result = codegen.module.verify();

        if !result.is_ok() {
            panic!("Module verification failed");
        }
    }

    #[test]
    fn test_if_else() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "test");

        let mut parser =
            Parser::new("let x = 5; if x > 3 { let y = 10 } else { let y = 20 }".to_string())
                .unwrap();
        let statements = parser.parse().unwrap();

        codegen.compile_statements(&statements).unwrap();

        let result = codegen.module.verify();

        dbg!(&result);
        if !result.is_ok() {
            dbg!(result.unwrap_err());
            panic!("Module verification failed");
        }
    }

    #[test]
    fn explicit_type_annotation() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "test");

        let mut parser = Parser::new("let x: i64 = 5;".to_string()).unwrap();
        let statements = parser.parse().unwrap();

        codegen.compile_statements(&statements).unwrap();

        let result = codegen.module.verify();

        dbg!(&result);
        if !result.is_ok() {
            dbg!(result.unwrap_err());
            panic!("Module verification failed");
        }
    }

    #[test]
    fn test_print_string() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context, "test_print");

        let mut parser = Parser::new("print \"Hello, World!\"".to_string()).unwrap();
        let statements = parser.parse().unwrap();

        codegen.compile_statements(&statements).unwrap();

        let result = codegen.module.verify();

        dbg!(&result);
        if !result.is_ok() {
            dbg!(result.unwrap_err());
            panic!("Module verification failed");
        }

        let ir_string = codegen.get_ir_string();
        assert!(ir_string.contains("@puts"));
        assert!(ir_string.contains("call i32 @puts"));
    }
}
