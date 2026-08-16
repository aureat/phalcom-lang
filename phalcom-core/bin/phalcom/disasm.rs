//! Recursive bytecode disassembler for Phalcom (IS §12, catalog §3).

use phalcom_core::bytecode::Bytecode;
use phalcom_core::chunk::Chunk;
use phalcom_core::error::PhError;
use phalcom_core::heap::{ObjRef, Object};
use phalcom_core::method::MethodKind;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::collections::HashSet;

/// Disassembles the compiled source code, printing its chunks recursively (IS §12).
pub fn disassemble_source(source: &str) -> Result<(), PhError> {
    let mut vm = VM::new();
    let module = vm.create_module("main", "<main>");
    let closure_ref = vm.compile_closure(module, source)?;

    // Track closures we have already disassembled to avoid duplicate disassembly.
    let mut visited = HashSet::new();
    visited.insert(closure_ref);

    let (name, chunk_to_disasm) = {
        let closure = vm.heap.closure(closure_ref);
        let name = vm.resolve_symbol(closure.callable.name_sym).to_string();
        let chunk = closure.callable.chunk.clone();
        (name, chunk)
    };

    let closure_info = {
        let closure = vm.heap.closure(closure_ref);
        (closure.callable.max_slots, closure.callable.num_upvalues)
    };
    println!("<main>   <main>   slots={} upvalues={}", closure_info.0, closure_info.1);
    disassemble_chunk(&mut vm, &chunk_to_disasm, 0, &mut visited, &name, source)?;
    Ok(())
}

fn disassemble_chunk(vm: &mut VM, chunk: &Chunk, indent: usize, visited: &mut HashSet<ObjRef>, parent_name: &str, source: &str) -> Result<(), PhError> {
    let indent_str = "  ".repeat(indent);

    println!("{}constants:", indent_str);
    for (i, constant) in chunk.constants.iter().enumerate() {
        let val_str = match constant {
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::Class(c) => format!("<class {}>", c.name),
                Object::Closure(cls) => {
                    let name = vm.resolve_symbol(cls.callable.name_sym).to_string();
                    format!("<closure {}>", name)
                }
                Object::Block(b) => {
                    let cls = vm.heap.closure(b.closure);
                    let name = vm.resolve_symbol(cls.callable.name_sym).to_string();
                    format!("<block in {}>", name)
                }
                Object::Method(m) => {
                    let sel = vm.resolve_symbol(m.signature.selector).to_string();
                    match m.kind {
                        MethodKind::Closure(_) => format!("<method {}>", sel),
                        MethodKind::Primitive(_) => format!("<primitive {}>", sel),
                    }
                }
                _ => format!("{:?}", constant),
            },
            Value::Symbol(sym) => format!("Symbol({})", vm.resolve_symbol(*sym)),
            _ => format!("{:?}", constant),
        };
        println!("{}  [{}] {}", indent_str, i, val_str);
    }

    // Resolve source for line number resolution
    let source_id = chunk.source_id;
    let main_sym = vm.get_or_intern("main");
    let main_module_ref = vm.find_module_by_symbol(main_sym);
    let source_text = main_module_ref
        .and_then(|m| vm.heap.module(m).source_at(source_id).map(|s| s.as_str()))
        .unwrap_or(source);

    let mut pending_nested = Vec::new();

    // Scan for nested closures:
    // (a) `Closure` bytecode instructions referencing a constant index.
    for instr in &chunk.code {
        if let Bytecode::Closure(idx) = *instr {
            let val = chunk.constants[idx as usize];
            if let Some(id) = val.as_obj() {
                if visited.insert(id) {
                    let cls = vm.heap.closure(id);
                    let name = vm.resolve_symbol(cls.callable.name_sym).to_string();
                    pending_nested.push((id, name, cls.callable.chunk.clone()));
                }
            }
        }
    }
    // (b) `Method` objects sitting directly in the constant pool (class method bodies).
    //     The compiler stores compiled methods as `Object::Method` constants and emits
    //     `Bytecode::Method` (not `Bytecode::Closure`) to attach them, so the loop above
    //     never sees them. Walk the constant pool explicitly here.
    for constant in &chunk.constants {
        if let Value::Obj(id) = *constant {
            if let Object::Method(m) = vm.heap.get(id) {
                if let MethodKind::Closure(body_ref) = m.kind {
                    if visited.insert(body_ref) {
                        let cls = vm.heap.closure(body_ref);
                        let name = vm.resolve_symbol(cls.callable.name_sym).to_string();
                        pending_nested.push((body_ref, name, cls.callable.chunk.clone()));
                    }
                }
            }
        }
    }

    println!("\n{}bytecode:", indent_str);
    let mut ip = 0;
    while ip < chunk.code.len() {
        let instr = chunk.code[ip];
        let line = chunk.line_at(ip, source_text);

        let instr_str = match instr {
            Bytecode::Invoke(arity, sel_idx) => {
                let sel_val = chunk.constants[sel_idx as usize];
                let sel_sym = sel_val.as_symbol().unwrap();
                format!("Invoke({}, {})", vm.resolve_symbol(sel_sym), arity)
            }
            Bytecode::InvokeLocal(slot, arity, sel_idx) => {
                let sel_val = chunk.constants[sel_idx as usize];
                let sel_sym = sel_val.as_symbol().unwrap();
                format!("InvokeLocal({}, {}, {})", slot, arity, vm.resolve_symbol(sel_sym))
            }
            Bytecode::InvokeConst(const_idx, arity, sel_idx) => {
                let sel_val = chunk.constants[sel_idx as usize];
                let sel_sym = sel_val.as_symbol().unwrap();
                format!("InvokeConst({}, {}, {})", const_idx, arity, vm.resolve_symbol(sel_sym))
            }
            Bytecode::Closure(idx) => {
                let val = chunk.constants[idx as usize];
                let mut capture_ann = String::new();
                if let Some(id) = val.as_obj() {
                    let cls = vm.heap.closure(id);
                    if !cls.callable.upvalues.is_empty() {
                        capture_ann = "        ← captures: ".to_string();
                        let names: Vec<String> = cls.callable.upvalues.iter().map(|desc| format!("{}", desc.index)).collect();
                        capture_ann.push_str(&names.join(", "));
                    }
                }
                format!("Closure(idx={}){}", idx, capture_ann)
            }
            _ => format!("{:?}", instr),
        };

        // Note: fused superinstructions leave a dead Invoke at ip + 1
        let is_fused = matches!(instr, Bytecode::InvokeLocal(..) | Bytecode::InvokeConst(..));

        println!("{}  {:04}  line {}   {}", indent_str, ip, line, instr_str);

        if is_fused {
            // Print the shadowed dead Invoke slot at ip + 1
            ip += 1;
            let dead_instr = chunk.code[ip];
            println!("{}  {:04}  line {}   [shadowed dead slot] {:?}", indent_str, ip, line, dead_instr);
        }

        ip += 1;
    }

    // Now recurse into nested closures
    for (id, name, nested_chunk) in pending_nested {
        println!();
        let prefix = "  ".repeat(indent);
        let closure = vm.heap.closure(id);

        let block_prefix = if name.contains("<block") || name.is_empty() || name == parent_name {
            format!("<block in {}>", parent_name)
        } else {
            format!("{}.{}", parent_name, name)
        };

        println!(
            "{}└─ {}   slots={} upvalues={}",
            prefix, block_prefix, closure.callable.max_slots, closure.callable.num_upvalues
        );
        disassemble_chunk(vm, &nested_chunk, indent + 3, visited, &block_prefix, source)?;
    }

    Ok(())
}
