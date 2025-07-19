use phalcom_vm::interner::Symbol;
use phalcom_ast::ast::{BinaryOp, Expr, Program, Statement, UnaryOp};
use phalcom_common::{phref_new, PhRef};
use phalcom_vm::bytecode::Bytecode;
use phalcom_vm::chunk::Chunk;
use phalcom_vm::closure::ClosureObject;
// use phalcom_ast::parser::Parser; // Not present, use lalrpop_util parser directly
use phalcom_vm::error::PhError;
use phalcom_vm::module::ModuleObject;
use phalcom_vm::value::Value;
use phalcom_vm::vm::VM;
use thiserror::Error;

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
    #[error("{0}")]
    Message(String),
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

    fn compile_block(
        &mut self,
        statements: Vec<Statement>,
        name_sym: Symbol,
        arity: usize,
        _is_static_method: bool,
    ) -> Result<PhRef<ClosureObject>, CompilerError> {
        let mut block_compiler = Compiler {
            vm: self.vm,
            module: self.module.clone(),
            chunk: Chunk::default(),
        };
        let len = statements.len();
        let mut last_is_return = false;
        for (i, statement) in statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
            }
            block_compiler.compile_statement_with_pop_control(statement, !is_last)?;
        }
        if !last_is_return {
            block_compiler.chunk.add_instruction(Bytecode::Return);
        }

        let callable = phalcom_vm::callable::Callable {
            chunk: block_compiler.chunk,
            max_slots: 0, // TODO: Calculate max_slots
            num_upvalues: 0, // TODO: Calculate num_upvalues
            arity,
            name_sym,
        };
        let closure = phref_new(ClosureObject {
            callable,
            module: self.module.clone(),
            upvalues: Vec::new(),
        });
        Ok(closure)
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
                }
                else {
                    self.chunk.add_instruction(Bytecode::Nil);
                }
                self.chunk.add_instruction(Bytecode::Return);
            }
            Statement::Class(class_def) => {
                // Push superclass onto the stack (for now, default to Object)
                let object_class = self.vm.universe.classes.object_class.clone();
                let superclass_idx = self.chunk.add_constant(Value::Class(object_class));
                self.chunk.add_instruction(Bytecode::Constant(superclass_idx));
                // TODO: Handle explicit superclass syntax later

                let name_sym = self.vm.interner.intern(&class_def.name);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::Class(name_idx));

                // The class object is now on top of the stack. Iterate through members.
                for member in class_def.members {
                    match member {
                        phalcom_ast::ast::ClassMember::Method(method_def) => {
                            println!("[Compiler] Compiling method: {} (static: {})", method_def.name, method_def.is_static);
                            let method_name_sym = self.vm.interner.intern(&method_def.name);
                            let arity = method_def.params.len();
                            let closure = self.compile_block(method_def.body, method_name_sym, arity, method_def.is_static)?;

                            let method_obj = phref_new(phalcom_vm::method::MethodObject::new(
                                method_name_sym,
                                phalcom_vm::method::SignatureKind::Method(arity as u8),
                                phalcom_vm::method::MethodKind::Closure(closure),
                            ));

                            let method_obj_idx = self.chunk.add_constant(Value::Method(method_obj));
                            println!("[Compiler] Emitting Constant for method_obj_idx: {}", method_obj_idx);
                            self.chunk.add_instruction(Bytecode::Constant(method_obj_idx));
                            let selector_idx = self.chunk.add_constant(Value::Symbol(method_name_sym));
                            println!("[Compiler] Emitting Method for selector_idx: {}, is_static: {}", selector_idx, method_def.is_static);
                            self.chunk.add_instruction(Bytecode::Method(selector_idx, method_def.is_static));
                        }
                        phalcom_ast::ast::ClassMember::Getter(getter_def) => {
                            println!("[Compiler] Compiling getter: {} (static: {})", getter_def.name, getter_def.is_static);
                            if getter_def.is_static {
                                return Err(CompilerError::Message("Static getters are not allowed.".to_string()));
                            }
                            let getter_name_sym = self.vm.interner.intern(&getter_def.name);
                            let closure = self.compile_block(getter_def.body, getter_name_sym, 0, getter_def.is_static)?;

                            let method_obj = phref_new(phalcom_vm::method::MethodObject::new(
                                getter_name_sym,
                                phalcom_vm::method::SignatureKind::Getter,
                                phalcom_vm::method::MethodKind::Closure(closure),
                            ));

                            let method_obj_idx = self.chunk.add_constant(Value::Method(method_obj));
                            self.chunk.add_instruction(Bytecode::Constant(method_obj_idx));
                            let selector_idx = self.chunk.add_constant(Value::Symbol(getter_name_sym));
                            self.chunk.add_instruction(Bytecode::Method(selector_idx, getter_def.is_static));
                        }
                        phalcom_ast::ast::ClassMember::Setter(setter_def) => {
                            println!("[Compiler] Compiling setter: {} (static: {})", setter_def.name, setter_def.is_static);
                            if setter_def.is_static {
                                return Err(CompilerError::Message("Static setters are not allowed.".to_string()));
                            }
                            let setter_name_sym = self.vm.interner.intern(&setter_def.name);
                            let closure = self.compile_block(setter_def.body, setter_name_sym, 1, setter_def.is_static)?;

                            let method_obj = phref_new(phalcom_vm::method::MethodObject::new(
                                setter_name_sym,
                                phalcom_vm::method::SignatureKind::Setter,
                                phalcom_vm::method::MethodKind::Closure(closure),
                            ));

                            let method_obj_idx = self.chunk.add_constant(Value::Method(method_obj));
                            self.chunk.add_instruction(Bytecode::Constant(method_obj_idx));
                            let selector_idx = self.chunk.add_constant(Value::Symbol(setter_name_sym));
                            self.chunk.add_instruction(Bytecode::Method(selector_idx, setter_def.is_static));
                        }
                    }
                }

                // After defining all methods, the class is still on the stack.
                // Define it as a global variable.
                self.chunk.add_instruction(Bytecode::DefineGlobal(name_idx));
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        match expr {
            Expr::SetProperty(set_prop) => {
                self.compile_expr(set_prop.object)?;
                self.compile_expr(set_prop.value)?;
                let name_sym = self.vm.interner.intern(&set_prop.property);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::SetProperty(name_idx));
            }
                        Expr::MethodCall(method_call) => {
                self.compile_expr(method_call.object)?;
                for arg in &method_call.args {
                    self.compile_expr(arg.clone())?;
                }
                let selector_sym = self.vm.interner.intern(&method_call.method);
                let selector_idx = self.chunk.add_constant(Value::Symbol(selector_sym));
                self.chunk.add_instruction(Bytecode::Invoke(method_call.args.len() as u8, selector_idx));
            }
            Expr::GetProperty(get_prop) => {
                self.compile_expr(get_prop.object)?;
                let name_sym = self.vm.interner.intern(&get_prop.property);
                let name_idx = self.chunk.add_constant(Value::Symbol(name_sym));
                self.chunk.add_instruction(Bytecode::GetProperty(name_idx));
            }
            Expr::Number(n) => {
                let idx = self.chunk.add_constant(Value::Number(n));
                self.chunk.add_instruction(Bytecode::Constant(idx));
            }
            Expr::String(s) => {
                let string_obj = Value::string_from(s);
                let idx = self.chunk.add_constant(string_obj);
                self.chunk.add_instruction(Bytecode::Constant(idx));
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
            Expr::SelfVar => {
                // TODO: Handle `self` keyword. For now, push Nil.
                self.chunk.add_instruction(Bytecode::Nil);
            }
            Expr::SuperVar => {
                // TODO: Handle `super` keyword. For now, push Nil.
                self.chunk.add_instruction(Bytecode::Nil);
            }
            Expr::Call(call_expr) => {
                // TODO: Implement function call compilation
                self.compile_expr(call_expr.callee)?;
                for arg in call_expr.args {
                    self.compile_expr(arg)?;
                }
                // For now, push Nil as a placeholder for the return value
                self.chunk.add_instruction(Bytecode::Nil);
            }
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
        let result = run_test("return 123.class.superclass;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
            _ => panic!("Expected Value::Class for 123.class.superclass"),
        }
        // String superclass
        let result = run_test("return \"abc\".class.superclass;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
            _ => panic!("Expected Value::Class for \"abc\".class.superclass"),
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
        let result = run_test("return 123.class.class;").unwrap();
        println!("{:?}", result);

        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Class"),
            _ => panic!("Expected Value::Class for 123.class.class"),
        }
        // .class.superclass should be Object
        let result = run_test("return 123.class.superclass;").unwrap();
        match result {
            Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
            _ => panic!("Expected Value::Class for 123.class.superclass"),
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
    fn test_compile_binary_mult() {
        let result = run_test("return 4 * 3;").unwrap();
        assert_eq!(result, Value::Number(12.0));
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

    #[test]
    fn test_compile_class_add_call() {
        let result = run_test("return 123.class + true.class;").unwrap();
        println!("{:?}", result);
    }
}
