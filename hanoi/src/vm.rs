use typed_index_collections::TiSliceIndex;

use crate::{
    ast,
    flat::{Builtin, EntryView, Library, NamespaceRef, Pointer, Value, Word},
};

fn eval(lib: &Library, stack: &mut Vec<Value>, w: &Word) {
    match w {
        Word::Builtin(Builtin::Add) => {
            let Some(Value::Usize(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Usize(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Usize(a + b));
        }
        Word::Copy(idx) => {
            stack.push(stack[stack.len() - idx - 1].clone());
        }
        Word::Move(idx) => {
            let val = stack.remove(stack.len() - idx - 1);
            stack.push(val);
        }
        Word::Drop(idx) => {
            stack.remove(stack.len() - idx - 1);
        }
        Word::Push(v) => stack.push(v.clone()),
        Word::Builtin(Builtin::Eq) => {
            let Some(a) = stack.pop() else {
                panic!("bad value")
            };
            let Some(b) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a == b));
        }
        Word::Builtin(Builtin::Curry) => {
            let (mut closure, code) = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(val) = stack.pop() else { panic!() };
            closure.insert(0, val);
            stack.push(Value::Pointer(closure, code.idx));
        }
        Word::Builtin(Builtin::IsCode) => {
            let value = Value::Bool(match stack.pop().unwrap() {
                Value::Pointer(_, _) => true,
                _ => false,
            });
            stack.push(value)
        }
        Word::Builtin(Builtin::And) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a && b));
        }
        Word::Builtin(Builtin::Or) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a || b));
        }
        Word::Builtin(Builtin::Not) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(!a));
        }
        Word::Builtin(Builtin::Get) => {
            let Some(Value::Namespace(ns_idx)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Symbol(name)) = stack.pop() else {
                panic!("bad value")
            };
            let ns = NamespaceRef { lib, idx: ns_idx };

            stack.push(match ns.get(&name).unwrap() {
                crate::flat::EntryView::Code(code) => Value::Pointer(vec![], code.idx),
                crate::flat::EntryView::Namespace(ns) => Value::Namespace(ns.idx),
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct Arena {
    pub buffers: Vec<Buffer>,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    mem: Vec<usize>,
}

fn control_flow(
    lib: &Library,
    stack: &mut Vec<Value>,
    arena: &mut Arena,
) -> Option<Vec<(Word, Pointer)>> {
    let Some(Value::Symbol(op)) = stack.pop() else {
        panic!("bad value")
    };
    let (push, next) = match op.as_str() {
        // "malloc" => {
        //     let Some(Value::Usize(size)) = stack.pop() else {
        //         panic!()
        //     };
        //     let next = stack.pop().unwrap().into_code(lib).unwrap();

        //     let handle = Value::Handle(arena.buffers.len());

        //     arena.buffers.push(Buffer { mem: vec![0; size] });

        //     stack.push(handle);
        //     Some(next.into_words())
        // }
        // "set_mem" => {
        //     let Some(Value::Handle(handle)) = stack.pop() else {
        //         panic!()
        //     };
        //     let Some(Value::Usize(offset)) = stack.pop() else {
        //         panic!()
        //     };
        //     let Some(Value::Usize(value)) = stack.pop() else {
        //         panic!()
        //     };
        //     let next = stack.pop().unwrap().into_code(lib).unwrap();

        //     let buf = arena.buffers.get_mut(handle).unwrap();
        //     buf.mem[offset] = value;

        //     Some(next.into_words())
        // }
        // "get_mem" => {
        //     let Some(Value::Handle(handle)) = stack.pop() else {
        //         panic!()
        //     };
        //     let Some(Value::Usize(offset)) = stack.pop() else {
        //         panic!()
        //     };
        //     let next = stack.pop().unwrap().into_code(lib).unwrap();

        //     let buf = arena.buffers.get_mut(handle).unwrap();
        //     stack.push(Value::Usize(buf.mem[offset]));

        //     Some(next.into_words())
        // }
        "if" => {
            let false_case = stack.pop().unwrap().into_code(lib).unwrap();
            let true_case = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(Value::Bool(cond)) = stack.pop() else {
                panic!()
            };
            if cond {
                Some(true_case)
            } else {
                Some(false_case)
            }
        }
        "exec" => Some(stack.pop().unwrap().into_code(lib).unwrap()),
        "assert" => None,
        // "halt" => None,
        unk => panic!("unknown symbol: {}", unk),
    }?;

    stack.extend(push);
    Some(next.words())
}

pub struct Vm {
    pub lib: Library,
    pub prog: Vec<(Word, Pointer)>,
    pub stack: Vec<Value>,
    pub arena: Arena,
}

impl Vm {
    pub fn new(ast: ast::Namespace) -> Self {
        let lib = Library::from_ast(ast);

        let EntryView::Code(main) = lib.root_namespace().get("main").unwrap() else {
            panic!("not code")
        };

        let prog = main.words().into_iter().rev().collect();
        Vm {
            lib,
            prog,
            stack: vec![],
            arena: Arena { buffers: vec![] },
        }
    }

    pub fn step(&mut self) -> bool {
        if let Some((word, ptr)) = self.prog.pop() {
            // eprintln!("word: {:?}", word);
            eval(&self.lib, &mut self.stack, &word);
            true
        } else {
            if let Some(new_prog) = control_flow(&self.lib, &mut self.stack, &mut self.arena) {
                self.prog = new_prog.into_iter().rev().collect();
                true
            } else {
                false
            }
        }
    }
}
