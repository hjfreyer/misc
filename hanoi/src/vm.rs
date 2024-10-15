use typed_index_collections::TiSliceIndex;

use crate::{
    ast,
    flat::{
        Builtin, CodeIndex, CodeRef, CodeView, EntryView, InnerWord, Library, NamespaceRef, Value,
        Word,
    },
};

fn eval(lib: &Library, stack: &mut Vec<Value>, w: &InnerWord) {
    match w {
        InnerWord::Builtin(Builtin::Add) => {
            let Some(Value::Usize(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Usize(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Usize(a + b));
        }
        InnerWord::Copy(idx) => {
            stack.push(stack[stack.len() - idx - 1].clone());
        }
        InnerWord::Move(idx) => {
            let val = stack.remove(stack.len() - idx - 1);
            stack.push(val);
        }
        InnerWord::Drop(idx) => {
            stack.remove(stack.len() - idx - 1);
        }
        InnerWord::Push(v) => stack.push(v.clone()),
        InnerWord::Builtin(Builtin::Eq) => {
            let Some(a) = stack.pop() else {
                panic!("bad value")
            };
            let Some(b) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a == b));
        }
        InnerWord::Builtin(Builtin::AssertEq) => {
            let Some(b) = stack.pop() else {
                panic!("bad value")
            };
            let Some(a) = stack.pop() else {
                panic!("bad value")
            };
            assert_eq!(a, b);
        }
        InnerWord::Builtin(Builtin::Curry) => {
            let (mut closure, code) = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(val) = stack.pop() else { panic!() };
            closure.insert(0, val);
            stack.push(Value::Pointer(closure, code));
        }
        InnerWord::Builtin(Builtin::IsCode) => {
            let value = Value::Bool(match stack.pop().unwrap() {
                Value::Pointer(_, _) => true,
                _ => false,
            });
            stack.push(value)
        }
        InnerWord::Builtin(Builtin::And) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a && b));
        }
        InnerWord::Builtin(Builtin::Or) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a || b));
        }
        InnerWord::Builtin(Builtin::Not) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(!a));
        }
        InnerWord::Builtin(Builtin::Get) => {
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
        InnerWord::Builtin(Builtin::SymbolCharAt) => {
            let Some(Value::Usize(idx)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Symbol(sym)) = stack.pop() else {
                panic!("bad value")
            };

            stack.push(sym.chars().nth(idx).unwrap().into());
        }
        InnerWord::Builtin(Builtin::SymbolLen) => {
            let Some(Value::Symbol(sym)) = stack.pop() else {
                panic!("bad value")
            };

            stack.push(sym.chars().count().into());
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

fn control_flow<'t>(
    lib: &Library<'t>,
    stack: &mut Vec<Value>,
    arena: &mut Arena,
) -> Option<Vec<Word<'t>>> {
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
        "exec" => {
            let (push, code) = stack.pop().unwrap().into_code(lib).unwrap();
            if code == CodeIndex::TRAP {
                None
            } else {
                Some((push, code))
            }
        }
        "assert" => None,
        // "halt" => None,
        unk => panic!("unknown symbol: {}", unk),
    }?;

    stack.extend(push);
    Some(CodeRef { lib, idx: next }.words())
}

pub struct Vm<'t> {
    pub lib: Library<'t>,
    pub prog: Vec<Word<'t>>,
    pub stack: Vec<Value>,
    pub arena: Arena,
}

impl<'t> Vm<'t> {
    pub fn new(ast: ast::Namespace<'t>) -> Self {
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
        if let Some(word) = self.prog.pop() {
            // eprintln!("word: {:?}", word);
            eval(&self.lib, &mut self.stack, &word.inner);
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
