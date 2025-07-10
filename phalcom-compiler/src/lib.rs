

use phalcom_ast::{Program, Statement, Expr, BinaryOp, UnaryOp};
use phalcom_vm::bytecode::Bytecode;
use phalcom_vm::chunk::Chunk;
use phalcom_vm::closure::ClosureObject;
use phalcom_vm::module::ModuleObject;
use phalcom_vm::vm::VM;
use phalcom_vm::value::Value;
use phalcom_common::PhRef;
use thiserror::Error;
use phalcom_ast::parser::Parser;
use phalcom_vm::error::PhError;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Unknown error during compilation.")]
    Unknown,
    #[error("Undefined variable '{0}'.")]
    UndefinedVariable(String),
    #[error("Invalid assignment target.")]
    InvalidAssignmentTarget,
    #[error(transparent)]
    ParseError(#[from] lalrpop_util::ParseError<usize, phalcom_ast::token::Token, phalcom_ast::error::LexicalError>),
}

impl From<CompilerError> for PhError {
    fn from(err: CompilerError) -> Self {
        PhError::CompileError(err.to_string())
    }
}

pub fn compile(vm: &mut VM, source: &str) -> Result<PhRef<ClosureObject>, PhError> {
    let parser = Parser::new();
    let program = parser.parse(source).map_err(CompilerError::from)?;
    let module = vm.module_from_str("<main>");
    let compiler = Compiler::new(vm, module);
    let closure = compiler.compile(program)?;
    Ok(closure)
}

struct Compiler<'vm> {
    vm: &'vm mut VM,
    module: PhRef<ModuleObject>,
    chunk: Chunk,
}

impl<'vm> Compiler<'vm> {
    fn new(vm: &'vm mut VM, module: PhRef<ModuleObject>) -> Self {
        Compiler {
            vm,
            module,
            chunk: Chunk::default(),
        }
    }

    fn compile(mut self, program: Program) -> Result<PhRef<ClosureObject>, CompilerError> {
        for statement in program.statements {
            self.compile_statement(statement)?;
        }
        self.chunk.add_instruction(Bytecode::Return);

        let main_sym = self.vm.get_or_intern("<main>");

        let closure = PhRef::new(ClosureObject::new(
            phalcom_vm::callable::Callable::new(self.chunk, main_sym, 0),
            self.module.clone(),
        ));
        Ok(closure)
    }

    fn compile_statement(&mut self, statement: Statement) -> Result<(), CompilerError> {
        match statement {
            Statement::Expr(expr) => {
                self.compile_expr(expr)?;
                self.chunk.add_instruction(Bytecode::Pop);
            }
            Statement::Let(binding) => {
                if let Some(expr) = binding.value {
                    self.compile_expr(expr)?;
                } else {
                    self.chunk.add_instruction(Bytecode::Nil);
                }
                let name_sym = self.vm.get_or_intern(&binding.name);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::DefineGlobal(name_idx));
            }
            Statement::Return(return_stmt) => {
                if let Some(expr) = return_stmt.value {
                    self.compile_expr(expr)?;
                } else {
                    self.chunk.add_instruction(Bytecode::Nil);
                }
                self.chunk.add_instruction(Bytecode::Return);
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        match expr {
            Expr::Number(n) => {
                let idx = self.chunk.add_constant(Value::Number(n));
                self.chunk.add_instruction(Bytecode::Number(idx));
            }
            Expr::String(s) => {
                let string_obj = Value::string_from(s);
                let idx = self.chunk.add_constant(string_obj);
                self.chunk.add_instruction(Bytecode::String(idx));
            }
            Expr::Boolean(b) => {
                if b {
                    self.chunk.add_instruction(Bytecode::True);
                } else {
                    self.chunk.add_instruction(Bytecode::False);
                }
            }
            Expr::Nil => {
                self.chunk.add_instruction(Bytecode::Nil);
            }
            Expr::Var(name) => {
                let name_sym = self.vm.get_or_intern(&name);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::GetGlobal(name_idx));
            }
            Expr::Assignment(assign_expr) => {
                self.compile_expr(assign_expr.value)?;
                if let Expr::Var(name) = *assign_expr.name {
                    let name_sym = self.vm.get_or_intern(&name);
                    let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                    self.chunk.add_instruction(Bytecode::SetGlobal(name_idx));
                } else {
                    return Err(CompilerError::InvalidAssignmentTarget);
                }
            }
            Expr::Binary(binary_expr) => {
                self.compile_expr(binary_expr.left)?;
                self.compile_expr(binary_expr.right)?;
                match binary_expr.op {
                    BinaryOp::Add => self.chunk.add_instruction(Bytecode::Add),
                    BinaryOp::Subtract => self.chunk.add_instruction(Bytecode::Subtract),
                    BinaryOp::Multiply => self.chunk.add_instruction(Bytecode::Multiply),
                    BinaryOp::Divide => self.chunk.add_instruction(Bytecode::Divide),
                    BinaryOp::Modulo => self.chunk.add_instruction(Bytecode::Modulo),
                    BinaryOp::Equal => self.chunk.add_instruction(Bytecode::Equal),
                    BinaryOp::NotEqual => self.chunk.add_instruction(Bytecode::NotEqual),
                    BinaryOp::LessThan => self.chunk.add_instruction(Bytecode::Less),
                    BinaryOp::LessThanOrEqual => self.chunk.add_instruction(Bytecode::LessEqual),
                    BinaryOp::GreaterThan => self.chunk.add_instruction(Bytecode::Greater),
                    BinaryOp::GreaterThanOrEqual => self.chunk.add_instruction(Bytecode::GreaterEqual),
                    BinaryOp::And => self.chunk.add_instruction(Bytecode::And),
                    BinaryOp::Or => self.chunk.add_instruction(Bytecode::Or),
                }
            }
            Expr::Unary(unary_expr) => {
                self.compile_expr(unary_expr.expr)?;
                match unary_expr.op {
                    UnaryOp::Negate => self.chunk.add_instruction(Bytecode::Negate),
                    UnaryOp::Not => self.chunk.add_instruction(Bytecode::Not),
                }
            }
            _ => unimplemented!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_vm::value::Value;

    fn run_test(source: &str) -> Result<Value, PhError> {
        let mut vm = VM::new();
        let closure = compile(&mut vm, source)?;
        let module = vm.module_from_str("<main>");
        vm.run_module(module, closure)
    }

    #[test]
    fn test_compile_number() {
        let result = run_test("123;").unwrap();
        assert_eq!(result, Value::Number(123.0));
    }

    #[test]
    fn test_compile_string() {
        let result = run_test("\"hello\";").unwrap();
        assert_eq!(result, Value::string_from("hello".to_string()));
    }

    #[test]
    fn test_compile_boolean() {
        let result = run_test("true;").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = run_test("false;").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_compile_nil() {
        let result = run_test("nil;").unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_compile_binary_expr() {
        let result = run_test("1 + 2;").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_compile_unary_expr() {
        let result = run_test("-10;").unwrap();
        assert_eq!(result, Value::Number(-10.0));
    }

    #[test]
    fn test_compile_global_let() {
        let result = run_test("let a = 10; a;").unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_compile_global_assignment() {
        let result = run_test("let a = 10; a = 20; a;").unwrap();
        assert_eq!(result, Value::Number(20.0));
    }
}
