#![allow(unused)]

#[derive(Debug, Clone, PartialEq, Eq)]
enum Word {
    Add,
    Push(Value),
    Cons,
    Snoc,
    Eq,
    Copy(usize),
    Drop(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Symbol(&'static str),
    Usize(usize),
    List(Vec<Value>),
    Quote(Box<Sentence>),
    Handle(usize),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Sentence(Vec<Word>);

impl Sentence {
    fn push(&mut self, s: impl Into<Sentence>) {
        for w in s.into().0 {
            self.0.push(w)
        }
    }
}

fn eval(stack: &mut Vec<Value>, w: Word) {
    match w {
        Word::Add => {
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
        Word::Drop(idx) => {
            stack.remove(stack.len() - idx - 1);
        }
        Word::Push(v) => stack.push(v),
        Word::Cons => {
            let car = stack.pop().unwrap();
            let Some(Value::List(mut cdr)) = stack.pop() else {
                panic!()
            };
            cdr.push(car);
            stack.push(Value::List(cdr));
        }
        Word::Snoc => {
            let Some(Value::List(mut list)) = stack.pop() else {
                panic!()
            };
            let car = list.pop().unwrap();
            stack.push(Value::List(list));
            stack.push(car);
        }
        Word::Eq => {
            let Some(a) = stack.pop() else {
                panic!("bad value")
            };
            let Some(b) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a == b));
        }
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

impl From<usize> for Word {
    fn from(value: usize) -> Self {
        Self::Push(value.into())
    }
}

impl From<usize> for Sentence {
    fn from(value: usize) -> Self {
        Self(vec![value.into()])
    }
}

macro_rules! phrase {
    (copy($idx:expr)) => {
        Sentence(vec![Word::Copy($idx)])
    };
    (drop($idx:expr)) => {
        Sentence(vec![Word::Drop($idx)])
    };
    ($name:ident) => {
        Sentence::from($name.clone())
    };
    ($name:expr) => {
        Sentence::from($name.clone())
    };
}

macro_rules! sentcat {
    ($($part:expr,)*) => {{
        let mut res = Sentence(vec![]);
        $(res.push($part);)*
        res
    }};
}

impl From<Word> for Sentence {
    fn from(value: Word) -> Self {
        {
            let w = value;
            Sentence(vec![w])
        }
    }
}

macro_rules! paragraph {
    (@sent ()) => {
        sentcat![]
    };
    (@sent ({$($quote:tt)*} $($tail:tt)*)) => {
        sentcat![Word::Push(Value::Quote(Box::new(paragraph!($($quote)*)))), paragraph!(@sent ($($tail)*)), ]
    };
    (@sent ($flike:ident($($head:tt)*) $($tail:tt)*)) => {
        sentcat![phrase!($flike($($head)*)), paragraph!(@sent ($($tail)*)), ]
    };
    (@sent ($head:tt $($tail:tt)*)) => {
        sentcat![phrase!($head), paragraph!(@sent ($($tail)*)), ]
    };
    (@para (if { $($cond:tt)* } {$($true_case:tt)*} else {$($false_case:tt)*}) ()) => {
        sentcat![
            paragraph!($($cond)*),
            Word::Push(Value::Quote(Box::new(paragraph!($($true_case)*)))),
            Word::Push(Value::Quote(Box::new(paragraph!($($false_case)*)))),
            Word::Push(Value::Symbol("if")), ]
    };
    (@para ($($a:tt)*) ()) => {
        paragraph!(@sent ($($a)*))
    };
    (@para ($($a:tt)*) (; $($tail:tt)*)) => {
        sentcat![Word::Push(Value::Quote(Box::new(paragraph!($($tail)*)))), paragraph!(@sent ($($a)*)), ]
    };
    (@para ($($a:tt)*) ($head:tt $($tail:tt)*)) => {
        paragraph!(@para ($($a)* $head) ($($tail)*))
    };
    ($($tail:tt)*) => {
        paragraph!(@para () ($($tail)*))
    };
}

#[derive(Debug, Clone)]
struct Arena {
    buffers: Vec<Buffer>,
}

#[derive(Debug, Clone)]
struct Buffer {
    mem: Vec<usize>,
}

fn main() {
    let add = {
        let w = Word::Add;
        Sentence(vec![w])
    };
    let cons = {
        let w = Word::Cons;
        Sentence(vec![w])
    };
    let snoc = {
        let w = Word::Snoc;
        Sentence(vec![w])
    };
    let eq = {
        let w = Word::Eq;
        Sentence(vec![w])
    };

    let halt = Word::Push(Value::Symbol("halt"));
    let malloc = Word::Push(Value::Symbol("malloc"));
    let get_mem = Word::Push(Value::Symbol("get_mem"));
    let set_mem = Word::Push(Value::Symbol("set_mem"));
    let if_ = Word::Push(Value::Symbol("if"));

    let double = paragraph!(copy(0) add halt);

    let swap = paragraph!(copy(1) drop(2));

    let mut prog = paragraph!({{{double {halt} copy(1) 2 copy(4) set_mem} 1 copy(2) get_mem} 12 1 copy(3) set_mem} 4 malloc);

    let mut prog: Sentence = paragraph! {
        // 4
        4 malloc;
        12 1 copy(3) set_mem;
        1 copy(2) get_mem;
        if { double halt eq } {
            copy(1) 2 copy(4) set_mem;
            halt
        } else {
            halt
        }
    };
    let mut stack = vec![];
    let mut arena = Arena { buffers: vec![] };

    loop {
        for w in prog.0 {
            println!("{:?}", w);
            eval(&mut stack, w);
            println!("{:?}", stack);
            println!("");
        }
        match stack.pop().unwrap() {
            Value::Symbol(s) if s == "malloc" => {
                let Some(Value::Usize(size)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Quote(next)) = stack.pop() else {
                    panic!()
                };

                let handle = Value::Handle(arena.buffers.len());

                arena.buffers.push(Buffer { mem: vec![0; size] });

                stack.push(handle);
                prog = *next;
            }
            Value::Symbol(s) if s == "set_mem" => {
                let Some(Value::Handle(handle)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Usize(offset)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Usize(value)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Quote(next)) = stack.pop() else {
                    panic!()
                };

                let buf = arena.buffers.get_mut(handle).unwrap();
                buf.mem[offset] = value;

                prog = *next;
            }
            Value::Symbol(s) if s == "get_mem" => {
                let Some(Value::Handle(handle)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Usize(offset)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Quote(next)) = stack.pop() else {
                    panic!()
                };

                let buf = arena.buffers.get_mut(handle).unwrap();
                stack.push(Value::Usize(buf.mem[offset]));

                prog = *next;
            }
            Value::Symbol(s) if s == "if" => {
                let Some(Value::Quote(false_case)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Quote(true_case)) = stack.pop() else {
                    panic!()
                };
                let Some(Value::Bool(cond)) = stack.pop() else {
                    panic!()
                };
                if cond {
                    prog = *true_case;
                } else {
                    prog = *false_case;
                }
            }
            Value::Symbol(s) if s == "halt" => break,
            _ => panic!(),
        }
    }

    println!("{:?}", stack);
    println!("{:?}", arena);
}
