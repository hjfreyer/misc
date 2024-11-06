use std::{any, collections::VecDeque};

use anyhow::{bail, Context};
use pest::Span;
use thiserror::Error;
use typed_index_collections::TiSliceIndex;

use crate::{
    ast,
    flat::{Builtin, Closure, Entry, InnerWord, Library, Namespace2, SentenceIndex, Value, Word},
};

#[derive(Debug)]
pub struct EvalError<'t> {
    pub span: Option<Span<'t>>,
    pub source: anyhow::Error,
}

impl<'t> std::fmt::Display for EvalError<'t> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(span) = &self.span {
            let (line, col) = span.start_pos().line_col();
            write!(f, "at {}:{}: ", line, col)?;
        } else {
            write!(f, "at <unknown location>: ")?;
        }
        write!(f, "{}", self.source)
    }
}
impl<'t> std::error::Error for EvalError<'t> {}

macro_rules! eval_bail {
    ($span:expr, $fmt:expr) => {
       return Err(EvalError { span: $span, source: anyhow::anyhow!($fmt) })
    };

    ($span:expr, $fmt:expr, $($arg:tt)*) => {
        return Err(EvalError { span: $span, source: anyhow::anyhow!($fmt, $($arg)*) })
    };
}

fn eval<'t>(lib: &Library, stack: &mut Stack, w: &Word<'t>) -> Result<(), EvalError<'t>> {
    inner_eval(lib, stack, &w.inner).map_err(|e| EvalError {
        span: w.span,
        source: e,
    })
}

fn inner_eval(lib: &Library, stack: &mut Stack, w: &InnerWord) -> anyhow::Result<()> {
    match w {
        InnerWord::Builtin(Builtin::Add) => {
            let Some(Value::Usize(a)) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Usize(b)) = stack.pop() else {
                bail!("bad value")
            };
            stack.push(Value::Usize(a + b));
            Ok(())
        }
        InnerWord::Copy(idx) => stack.copy(*idx),
        InnerWord::Move(idx) => stack.mv(*idx),
        &InnerWord::Send(idx) => stack.sd(idx),
        &InnerWord::Drop(idx) => stack.drop(idx),
        InnerWord::Push(v) => {
            stack.push(v.clone());
            Ok(())
        }
        InnerWord::Builtin(Builtin::Eq) => {
            let Some(a) = stack.pop() else {
                bail!("bad value")
            };
            let Some(b) = stack.pop() else {
                bail!("bad value")
            };
            stack.push(Value::Bool(a == b));
            Ok(())
        }
        InnerWord::Builtin(Builtin::AssertEq) => {
            let Some(a) = stack.pop() else {
                bail!("bad value")
            };
            let Some(b) = stack.pop() else {
                bail!("bad value")
            };
            if a != b {
                bail!("assertion failed: {:?} != {:?}", a, b)
            }
            Ok(())
        }
        InnerWord::Builtin(Builtin::Curry) => {
            let Some(Value::Pointer(Closure(mut closure, code))) = stack.pop() else {
                bail!("bad value")
            };
            let Some(val) = stack.pop() else {
                bail!("bad value")
            };
            closure.insert(0, val);
            stack.push(Value::Pointer(Closure(closure, code)));
            Ok(())
        }
        InnerWord::Builtin(Builtin::And) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                bail!("bad value")
            };
            stack.push(Value::Bool(a && b));
            Ok(())
        }
        InnerWord::Builtin(Builtin::Or) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                bail!("bad value")
            };
            stack.push(Value::Bool(a || b));
            Ok(())
        }
        InnerWord::Builtin(Builtin::Not) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                bail!("bad value")
            };
            stack.push(Value::Bool(!a));
            Ok(())
        }
        InnerWord::Builtin(Builtin::Get) => {
            let ns_idx = match stack.pop() {
                Some(Value::Namespace(ns_idx)) => ns_idx,
                other => {
                    bail!("attempted to get from non-namespace: {:?}", other)
                }
            };
            let name = match stack.pop() {
                Some(Value::Symbol(name)) => name,
                other => {
                    bail!(
                        "attempted to index into namespace with non-symbol: {:?}",
                        other
                    )
                }
            };
            let ns = &lib.namespaces[ns_idx];

            let Some(entry) = ns.get(&name) else {
                bail!("unknown symbol: {}", name)
            };

            stack.push(match entry {
                crate::flat::Entry::Value(v) => v.clone(),
                crate::flat::Entry::Namespace(ns) => Value::Namespace(*ns),
            });
            Ok(())
        }
        InnerWord::Builtin(Builtin::SymbolCharAt) => {
            let Some(Value::Usize(idx)) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Symbol(sym)) = stack.pop() else {
                bail!("bad value")
            };

            stack.push(sym.chars().nth(idx).unwrap().into());
            Ok(())
        }
        InnerWord::Builtin(Builtin::SymbolLen) => {
            let Some(Value::Symbol(sym)) = stack.pop() else {
                bail!("bad value")
            };

            stack.push(sym.chars().count().into());
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsEmpty) => {
            stack.push(Value::Namespace2(Namespace2 { items: vec![] }));
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsInsert) => {
            let Some(val) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Symbol(symbol)) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Namespace2(mut ns)) = stack.pop() else {
                bail!("bad value")
            };
            assert!(!ns.items.iter().any(|(k, v)| *k == symbol));
            ns.items.push((symbol, val));

            stack.push(Value::Namespace2(ns));
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsRemove) => {
            let Some(Value::Symbol(symbol)) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Namespace2(mut ns)) = stack.pop() else {
                bail!("bad value")
            };
            let pos = ns.items.iter().position(|(k, v)| *k == symbol).unwrap();
            let (_, val) = ns.items.remove(pos);

            stack.push(Value::Namespace2(ns));
            stack.push(val);
            Ok(())
        }
        InnerWord::Builtin(Builtin::NsGet) => {
            let Some(Value::Symbol(symbol)) = stack.pop() else {
                bail!("bad value")
            };
            let Some(Value::Namespace2(ns)) = stack.pop() else {
                bail!("bad value")
            };
            let pos = ns.items.iter().position(|(k, v)| *k == symbol).unwrap();
            let (_, val) = ns.items[pos].clone();

            stack.push(Value::Namespace2(ns));
            stack.push(val);
            Ok(())
        }
    }
}

fn control_flow<'t>(lib: &Library<'t>, stack: &mut Stack) -> anyhow::Result<Closure> {
    let Some(Value::Symbol(op)) = stack.pop() else {
        panic!("bad value")
    };
    match op.as_str() {
        // "malloc" => {
        //     let Some(Value::Usize(size)) = pop() else {
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
            let Some(Value::Pointer(false_case)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Pointer(true_case)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Bool(cond)) = stack.pop() else {
                panic!()
            };
            if cond {
                Ok(true_case)
            } else {
                Ok(false_case)
            }
        }
        "exec" => {
            let Some(Value::Pointer(next)) = stack.pop() else {
                panic!("bad value")
            };

            if !stack.is_empty() {
                bail!("exec with non-empty stack: {:?}", stack)
            }
            Ok(next)
        }
        // "halt" => None,
        unk => panic!("unknown symbol: {}", unk),
    }
}

pub struct Vm<'t> {
    pub lib: Library<'t>,
    pub pc: ProgramCounter,
    pub stack: Stack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramCounter {
    pub sentence_idx: SentenceIndex,
    pub word_idx: usize,
}

pub enum StepResult {
    Trap(Vec<Value>),
    Continue,
}

impl<'t> Vm<'t> {
    pub fn new(ast: ast::Namespace<'t>) -> Self {
        let lib = Library::from_ast(ast);

        let &Entry::Value(Value::Pointer(Closure(_, main))) =
            lib.root_namespace().get("main").unwrap()
        else {
            panic!("not code")
        };

        Vm {
            lib,
            pc: ProgramCounter {
                sentence_idx: main,
                word_idx: 0,
            },
            stack: Stack::default(),
        }
    }

    pub fn current_word(&self) -> Option<&Word<'t>> {
        self.lib.sentences[self.pc.sentence_idx]
            .words
            .get(self.pc.word_idx)
    }

    pub fn jump_to(&mut self, Closure(closure, sentence_idx): Closure) {
        for v in closure {
            self.stack.push(v);
        }
        self.pc.sentence_idx = sentence_idx;
        self.pc.word_idx = 0;
    }

    pub fn run_to_trap(&mut self) -> Result<Vec<Value>, EvalError<'t>> {
        loop {
            match self.step()? {
                StepResult::Continue => {}
                StepResult::Trap(t) => return Ok(t),
            }
        }
    }

    pub fn step(&mut self) -> Result<StepResult, EvalError<'t>> {
        let sentence = &self.lib.sentences[self.pc.sentence_idx];

        if let Some(word) = sentence.words.get(self.pc.word_idx) {
            eval(&self.lib, &mut self.stack, &word)?;
            self.pc.word_idx += 1;
            Ok(StepResult::Continue)
        } else {
            let next = control_flow(&self.lib, &mut self.stack).map_err(|e| EvalError {
                span: sentence.words.last().and_then(|w| w.span),
                source: e,
            })?;

            if next.1 == SentenceIndex::TRAP {
                Ok(StepResult::Trap(next.0))
            } else {
                self.jump_to(next);
                Ok(StepResult::Continue)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Stack {
    inner: Vec<Value>,
}

impl Stack {
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    pub fn push(&mut self, value: Value) {
        self.inner.push(value)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn copy(&mut self, back_idx: usize) -> anyhow::Result<()> {
        let Some(v) = self.inner.iter().rev().nth(back_idx) else {
            bail!("out of range: {}", back_idx)
        };

        self.push(v.clone());
        Ok(())
    }

    pub fn mv(&mut self, back_idx: usize) -> anyhow::Result<()> {
        let val = self.inner.remove(self.back_idx(back_idx)?);
        self.inner.push(val);
        Ok(())
    }

    pub fn sd(&mut self, back_idx: usize) -> anyhow::Result<()> {
        let new_idx = self.back_idx(back_idx)?;
        let Some(val) = self.inner.pop() else {
            bail!("bad value")
        };
        if self.inner.len() < new_idx {
            bail!("bad value")
        }
        self.inner.insert(new_idx, val);
        Ok(())
    }

    pub fn drop(&mut self, back_idx: usize) -> anyhow::Result<()> {
        self.inner.remove(self.back_idx(back_idx)?);
        Ok(())
    }

    fn back_idx(&self, back_idx: usize) -> anyhow::Result<usize> {
        self.inner
            .len()
            .checked_sub(1 + back_idx)
            .context("index out of range")
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Value> {
        self.inner.iter()
    }
}
