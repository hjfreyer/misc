use std::{any, collections::VecDeque};

use anyhow::bail;
use pest::Span;
use thiserror::Error;
use typed_index_collections::TiSliceIndex;

use crate::{
    ast,
    flat::{Builtin, Entry, InnerWord, Library, Namespace2, SentenceIndex, Value, Word},
};

#[derive(Error, Debug)]
#[error("at {span:?}: {source}")]
pub struct EvalError<'t> {
    pub span: Option<Span<'t>>,
    #[source]
    pub source: anyhow::Error,
}
macro_rules! eval_bail {
    ($span:expr, $fmt:expr) => {
        return Err(EvalError { span: $span, source: anyhow::anyhow!($fmt) })
    };

    ($span:expr, $fmt:expr, $($arg:tt)*) => {
        return Err(EvalError { span: $span, source: anyhow::anyhow!($fmt, $($arg)*) })
    };
}

fn eval<'t>(lib: &Library, stack: &mut VecDeque<Value>, w: &Word<'t>) -> Result<(), EvalError<'t>> {
    match &w.inner {
        InnerWord::Builtin(Builtin::Add) => {
            let Some(Value::Usize(a)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Usize(b)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            stack.push_front(Value::Usize(a + b));
            Ok(())
        }
        InnerWord::Copy(idx) => {
            let Some(get) = stack.get(*idx) else {
                eval_bail!(w.span, "bad value")
            };
            stack.push_front(get.clone());
            Ok(())
        }
        InnerWord::Move(idx) => {
            let Some(val) = stack.remove(*idx) else {
                eval_bail!(w.span, "bad value")
            };
            stack.push_front(val);
            Ok(())
        }
        &InnerWord::Send(idx) => {
            let Some(val) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            if stack.len() < idx {
                eval_bail!(w.span, "bad value")
            }
            stack.insert(idx, val);
            Ok(())
        }
        &InnerWord::Drop(idx) => {
            let Some(_) = stack.remove(idx) else {
                eval_bail!(w.span, "bad value")
            };
            Ok(())
        }
        InnerWord::Push(v) => {
            stack.push_front(v.clone());
            Ok(())
        }
        InnerWord::Builtin(Builtin::Eq) => {
            let Some(a) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(b) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            stack.push_front(Value::Bool(a == b));
            Ok(())
        }
        InnerWord::Builtin(Builtin::AssertEq) => {
            let Some(a) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(b) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            if a != b {
                eval_bail!(w.span, "assertion failed: {:?} != {:?}", a, b)
            }
            Ok(())
        }
        InnerWord::Builtin(Builtin::Curry) => {
            let Some(Value::Pointer(mut closure, code)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(val) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            closure.insert(0, val);
            stack.push_front(Value::Pointer(closure, code));
            Ok(())
        }
        InnerWord::Builtin(Builtin::And) => {
            let Some(Value::Bool(a)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Bool(b)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            stack.push_front(Value::Bool(a && b));
            Ok(())
        }
        InnerWord::Builtin(Builtin::Or) => {
            let Some(Value::Bool(a)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Bool(b)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            stack.push_front(Value::Bool(a || b));
            Ok(())
        }
        InnerWord::Builtin(Builtin::Not) => {
            let Some(Value::Bool(a)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            stack.push_front(Value::Bool(!a));
            Ok(())
        }
        InnerWord::Builtin(Builtin::Get) => {
            let Some(Value::Namespace(ns_idx)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Symbol(name)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let ns = &lib.namespaces[ns_idx];

            stack.push_front(match ns.get(&name).unwrap() {
                crate::flat::Entry::Value(v) => v.clone(),
                crate::flat::Entry::Namespace(ns) => Value::Namespace(*ns),
            });
            Ok(())
        }
        InnerWord::Builtin(Builtin::SymbolCharAt) => {
            let Some(Value::Usize(idx)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Symbol(sym)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };

            stack.push_front(sym.chars().nth(idx).unwrap().into());
            Ok(())
        }
        InnerWord::Builtin(Builtin::SymbolLen) => {
            let Some(Value::Symbol(sym)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };

            stack.push_front(sym.chars().count().into());
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsEmpty) => {
            stack.push_front(Value::Namespace2(Namespace2 { items: vec![] }));
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsInsert) => {
            let Some(val) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Symbol(symbol)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Namespace2(mut ns)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            assert!(!ns.items.iter().any(|(k, v)| *k == symbol));
            ns.items.push((symbol, val));

            stack.push_front(Value::Namespace2(ns));
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsRemove) => {
            let Some(Value::Symbol(symbol)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Namespace2(mut ns)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let pos = ns.items.iter().position(|(k, v)| *k == symbol).unwrap();
            let (_, val) = ns.items.remove(pos);

            stack.push_front(Value::Namespace2(ns));
            stack.push_front(val);
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsGet) => {
            let Some(Value::Symbol(symbol)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let Some(Value::Namespace2(ns)) = stack.pop_front() else {
                eval_bail!(w.span, "bad value")
            };
            let pos = ns.items.iter().position(|(k, v)| *k == symbol).unwrap();
            let (_, val) = ns.items[pos].clone();

            stack.push_front(Value::Namespace2(ns));
            stack.push_front(val);
            Ok(())
        }
    }
}

fn control_flow<'t>(
    lib: &Library<'t>,
    stack: &mut VecDeque<Value>,
) -> Result<Option<SentenceIndex>, EvalError<'t>> {
    let Some(Value::Symbol(op)) = stack.pop_front() else {
        panic!("bad value")
    };
    let res = match op.as_str() {
        // "malloc" => {
        //     let Some(Value::Usize(size)) = stack.pop_front() else {
        //         panic!()
        //     };
        //     let next = stack.pop().unwrap().into_code(lib).unwrap();

        //     let handle = Value::Handle(arena.buffers.len());

        //     arena.buffers.push(Buffer { mem: vec![0; size] });

        //     stack.push_front(handle);
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
        //     stack.push_front(Value::Usize(buf.mem[offset]));

        //     Some(next.into_words())
        // }
        "if" => {
            let false_case = stack.pop_front().unwrap().into_code(lib).unwrap();
            let true_case = stack.pop_front().unwrap().into_code(lib).unwrap();
            let Some(Value::Bool(cond)) = stack.pop_front() else {
                panic!()
            };
            if cond {
                Some(true_case)
            } else {
                Some(false_case)
            }
        }
        "exec" => {
            let (push, code) = stack.pop_front().unwrap().into_code(lib).unwrap();
            assert_eq!(stack, &vec![]);
            if code == SentenceIndex::TRAP {
                None
            } else {
                Some((push, code))
            }
        }
        "assert" => None,
        // "halt" => None,
        unk => panic!("unknown symbol: {}", unk),
    };

    if let Some((push, next)) = res {
        for v in push.into_iter() {
            stack.push_front(v);
        }
        Ok(Some(next))
    } else {
        Ok(None)
    }
}

pub struct Vm<'t> {
    pub lib: Library<'t>,
    pub pc: ProgramCounter,
    pub stack: VecDeque<Value>,
}

pub struct ProgramCounter {
    pub sentence_idx: SentenceIndex,
    pub word_idx: usize,
}

impl<'t> Vm<'t> {
    pub fn new(ast: ast::Namespace<'t>) -> Self {
        let lib = Library::from_ast(ast);

        let &Entry::Value(Value::Pointer(_, main)) = lib.root_namespace().get("main").unwrap()
        else {
            panic!("not code")
        };

        Vm {
            lib,
            pc: ProgramCounter {
                sentence_idx: main,
                word_idx: 0,
            },
            stack: VecDeque::new(),
        }
    }

    pub fn current_word(&self) -> Option<&Word<'t>> {
        self.lib.sentences[self.pc.sentence_idx]
            .words
            .get(self.pc.word_idx)
    }

    pub fn step(&mut self) -> Result<bool, EvalError<'t>> {
        let sentence = &self.lib.sentences[self.pc.sentence_idx];

        if let Some(word) = sentence.words.get(self.pc.word_idx) {
            // eprintln!("word: {:?}", word);
            eval(&self.lib, &mut self.stack, &word)?;
            self.pc.word_idx += 1;
            Ok(true)
        } else {
            if let Some(new_prog) = control_flow(&self.lib, &mut self.stack)? {
                self.pc.sentence_idx = new_prog;
                self.pc.word_idx = 0;
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
}
