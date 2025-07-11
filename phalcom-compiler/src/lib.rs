

use phalcom_ast::ast::{Program, Statement, Expr, BinaryOp, UnaryOp};
use phalcom_vm::bytecode::Bytecode;
use phalcom_vm::chunk::Chunk;
use phalcom_vm::closure::ClosureObject;
use phalcom_vm::module::ModuleObject;
use phalcom_vm::vm::VM;
use phalcom_vm::value::Value;
use phalcom_common::{phref_new, PhRef};
use thiserror::Error;
// use phalcom_ast::parser::Parser; // Not present, use lalrpop_util parser directly
use phalcom_vm::error::PhError;

use phalcom_ast::parser;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Unknown error during compilation.")]
    Unknown,
    #[error("Undefined variable '{0}'.")]
    UndefinedVariable(String),
    #[error("Invalid assignment target.")]
    InvalidAssignmentTarget,
    #[error("Parse error: {0:?}")]
    ParseError(lalrpop_util::ParseError<usize, phalcom_ast::token::Token, phalcom_ast::token::LexicalError>),
}


impl From<lalrpop_util::ParseError<usize, phalcom_ast::token::Token, phalcom_ast::token::LexicalError>> for CompilerError {
    fn from(err: lalrpop_util::ParseError<usize, phalcom_ast::token::Token, phalcom_ast::token::LexicalError>) -> Self {
        CompilerError::ParseError(err)
    }
}

impl From<CompilerError> for PhError {
    fn from(err: CompilerError) -> Self {
        PhError::StringError(err.to_string())
    }
}

pub fn compile(vm: &mut VM, source: &str) -> Result<PhRef<ClosureObject>, PhError> {
    let parser = phalcom_ast::parser::ProgramParser::new();
    let lexer = phalcom_ast::lexer::Lexer::new(source);
    let program = parser.parse(lexer).map_err(CompilerError::from)?;
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
        let len = program.statements.len();
        let mut last_is_return = false;
        for (i, statement) in program.statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            // Check if last statement is a return
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
            }
            self.compile_statement_with_pop_control(statement, !is_last)?;
        }
        if !last_is_return {
            self.chunk.add_instruction(Bytecode::Return);
        }

        let main_sym = self.vm.interner.intern("<main>");
        let callable = phalcom_vm::callable::Callable {
            chunk: self.chunk,
            max_slots: 0,
            num_upvalues: 0,
            arity: 0,
            name_sym: main_sym,
        };
        let closure = phref_new(ClosureObject {
            callable,
            module: self.module.clone(),
            upvalues: Vec::new(),
        });
        Ok(closure)
    }

    fn compile_statement(&mut self, statement: Statement) -> Result<(), CompilerError> {
        self.compile_statement_with_pop_control(statement, true)
    }

    fn compile_statement_with_pop_control(&mut self, statement: Statement, emit_pop: bool) -> Result<(), CompilerError> {
        match statement {
            Statement::Expr(expr) => {
                self.compile_expr(expr)?;
                if emit_pop {
                    self.chunk.add_instruction(Bytecode::Pop);
                }
            }
            Statement::Let(binding) => {
                if let Some(expr) = binding.value {
                    self.compile_expr(expr)?;
                } else {
                    self.chunk.add_instruction(Bytecode::Nil);
                }
                let name_sym = self.vm.interner.intern(&binding.name);
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
            Expr::GetProperty(get_prop) => {
                self.compile_expr(get_prop.object)?;
                let name_sym = self.vm.interner.intern(&get_prop.property);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::GetProperty(name_idx));
            }
            Expr::GetProperty(get_prop) => {
                self.compile_expr(get_prop.object)?;
                let name_sym = self.vm.interner.intern(&get_prop.property);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::GetProperty(name_idx));
            }
            Expr::GetProperty(get_prop) => {
                self.compile_expr(get_prop.object)?;
                let name_sym = self.vm.interner.intern(&get_prop.property);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::GetProperty(name_idx));
            }
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
                let name_sym = self.vm.interner.intern(&name);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::GetGlobal(name_idx));
            }
            Expr::Assignment(assign_expr) => {
                self.compile_expr(assign_expr.value)?;
                if let Expr::Var(name) = *assign_expr.name {
                    let name_sym = self.vm.interner.intern(&name);
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

    #[test]
    fn test_primitive_class() {
        // Number
        let result = run_test("return 123.class;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Number"),
            _ => panic!("Expected Value::Class for 123.class"),
        }
        // String
        let result = run_test("return \"abc\".class;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "String"),
            _ => panic!("Expected Value::Class for string.class"),
        }
        // Boolean
        let result = run_test("return true.class;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Bool"),
            _ => panic!("Expected Value::Class for true.class"),
        }
        // Nil
        let result = run_test("return nil.class;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Nil"),
            _ => panic!("Expected Value::Class for nil.class"),
        }
    }

    #[test]
    fn test_primitive_superclass() {
        // Number superclass
        let result = run_test("return 123.superclass;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
            _ => panic!("Expected Value::Class for 123.superclass"),
        }
        // String superclass
        let result = run_test("return \"abc\".superclass;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
            _ => panic!("Expected Value::Class for \"abc\".superclass"),
        }
    }

    #[test]
    fn test_primitive_name() {
        // Number name
        let result = run_test("return 123.name;").unwrap();
        println!("{:?}", result);

        match result {
            Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "Number"),
            _ => panic!("Expected Value::String for 123.name"),
        }
        // String name
        let result = run_test("\"abc\".name;").unwrap();
        match result {
            Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "String"),
            _ => panic!("Expected Value::String for string.name"),
        }
    }

    #[test]
    fn test_class_identity() {
        // .class.class should be Class
        let result = run_test("return 123.class;").unwrap();
        println!("{:?}", result);

        match result {
            Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "Class"),
            _ => panic!("Expected Value::String for 123.class.class"),
        }
        // .class.superclass should be Object
        let result = run_test("123.class.superclass;").unwrap();
        match result {
            Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "Object"),
            _ => panic!("Expected Value::String for 123.class.superclass"),
        }
    }
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
        let result = run_test("return true;").unwrap();
        assert_eq!(result, Value::Bool(true));
        let result = run_test("return false;").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_compile_nil() {
        let result = run_test("return nil;").unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_compile_binary_expr() {
        let result = run_test("return 1 + 2;").unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_compile_unary_expr() {
        let result = run_test("-10;").unwrap();
        assert_eq!(result, Value::Number(-10.0));
    }

    #[test]
    fn test_compile_global_let() {
        let result = run_test("let a = 10; return a;").unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_compile_global_assignment() {
        let result = run_test("let a = 10; a += 20; return a;").unwrap();
        assert_eq!(result, Value::Number(30.0));
    }

    #[test]
    fn test_complex_global_assignment() {
        let source = "
            let a = 5;
            let b = 10;
            a += b; // a should be 15
            return a;           // return 15
        ";
        let result = run_test(source).unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_compile_precedence() {
        let result = run_test("return 1 + 2 * 3;").unwrap();
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_compile_return() {
        let result = run_test("return 15; 20;").unwrap();
        assert_eq!(result, Value::Number(15.0));

        let result = run_test("return;").unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_compile_method_expr() {
        let result = run_test("return 123.class.name.class;").unwrap();
        println!("{:?}", result);
    }
}
