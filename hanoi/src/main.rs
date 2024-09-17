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
    Move(usize),
    Swap,
    Curry,
}

#[derive(Clone, PartialEq, Eq)]
enum Value {
    Symbol(&'static str),
    Usize(usize),
    List(Vec<Value>),
    Quote(Box<Sentence>),
    Handle(usize),
    Bool(bool),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Symbol(arg0) => write!(f, "'{}", arg0),
            Self::Usize(arg0) => write!(f, "{}", arg0),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Quote(arg0) => write!(f, "{{{:?}}}", arg0),
            Self::Handle(arg0) => f.debug_tuple("Handle").field(arg0).finish(),
            Self::Bool(arg0) => write!(f, "{}", arg0),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Sentence(Vec<Word>);

impl Sentence {
    fn push(&mut self, s: impl Into<Sentence>) {
        for w in s.into().0 {
            self.0.push(w)
        }
    }
}

impl std::fmt::Debug for Sentence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(Word::Push(Value::Quote(next))) = self.0.first() {
            let prefix = Sentence(self.0.iter().skip(1).cloned().collect());
            write!(f, "{:?}; {:?}", prefix, next)?;
        } else {
            for (i, w) in self.0.iter().enumerate() {
                if i != 0 {
                    write!(f, " ")?
                }
                match w {
                    Word::Add => write!(f, "add")?,
                    Word::Push(value) => write!(f, "{:?}", value)?,
                    Word::Cons => todo!(),
                    Word::Snoc => todo!(),
                    Word::Eq => write!(f, "eq")?,
                    Word::Copy(i) => write!(f, "copy({})", i)?,
                    Word::Drop(i) => write!(f, "drop({})", i)?,
                    Word::Move(i) => write!(f, "move({})", i)?,
                    Word::Swap => write!(f, "swap")?,
                    Word::Curry => write!(f, "curry")?,
                }
            }
        }

        Ok(())
    }
}

macro_rules! sentcat {
    ($($part:expr,)*) => {{
        let mut res = Sentence(vec![]);
        $(res.push($part);)*
        res
    }};
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
        Word::Move(idx) => {
            let val = stack.remove(stack.len() - idx - 1);
            stack.push(val);
        }
        Word::Drop(idx) => {
            stack.remove(stack.len() - idx - 1);
        }
        Word::Swap => {
            let a = stack.len() - 1;
            let b = a - 1;
            stack.swap(a, b);
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
        Word::Curry => {
            let Some(Value::Quote(code)) = stack.pop() else {
                panic!()
            };
            let Some(val) = stack.pop() else { panic!() };
            stack.push(Value::Quote(Box::new(sentcat![Word::Push(val), *code,])))
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
    (mv($idx:expr)) => {
        Sentence(vec![Word::Move($idx)])
    };
    ($name:ident) => {
        Sentence::from($name.clone())
    };
    ($name:expr) => {
        Sentence::from($name.clone())
    };
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

fn control_flow(stack: &mut Vec<Value>, arena: &mut Arena) -> Option<Sentence> {
    let Some(Value::Symbol(op)) = stack.pop() else {
        panic!("bad value")
    };
    match op {
        "malloc" => {
            let Some(Value::Usize(size)) = stack.pop() else {
                panic!()
            };
            let Some(Value::Quote(next)) = stack.pop() else {
                panic!()
            };

            let handle = Value::Handle(arena.buffers.len());

            arena.buffers.push(Buffer { mem: vec![0; size] });

            stack.push(handle);
            Some(*next)
        }
        "set_mem" => {
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

            Some(*next)
        }
        "get_mem" => {
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

            Some(*next)
        }
        "if" => {
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
                Some(*true_case)
            } else {
                Some(*false_case)
            }
        }
        "exec" => {
            let Some(Value::Quote(next)) = stack.pop() else {
                panic!()
            };
            Some(*next)
        }
        "halt" => None,
        _ => panic!(),
    }
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
    let curry = Word::Curry;

    let halt = Word::Push(Value::Symbol("halt"));
    let exec = Word::Push(Value::Symbol("exec"));
    let malloc = Word::Push(Value::Symbol("malloc"));
    let get_mem = Word::Push(Value::Symbol("get_mem"));
    let set_mem = Word::Push(Value::Symbol("set_mem"));
    let if_ = Word::Push(Value::Symbol("if"));
    let yield_ = Word::Push(Value::Symbol("yield"));
    let eos = Word::Push(Value::Symbol("eos"));
    let panic = Word::Push(Value::Symbol("panic"));

    let double = paragraph!(copy(0) add halt);

    let swap = Word::Swap;

    let inc = paragraph! { 1 add };

    let count: Sentence = paragraph! {
        // (caller next)
        1
        // (caller next 1)
        mv(2)
        // (next 1 caller)
        exec;
        2 mv(2) exec;
        3 mv(2) exec;
        eos
    };

    let evens_step = paragraph! {
        // (caller countnext i mynext)
        swap copy(0) add
        // (caller countnext mynext 2*i)
        mv(2) mv(2)
        // (caller 2*i countnext mynext)
        curry
        // (caller 2*i evensnext)
        mv(1) mv(2)
        // (evensnext 2*i caller)
        exec
    };

    let evens: Sentence = paragraph! {
        // (caller mynext)
        count;
        // (caller countnext 1 mynext)
        evens_step;
        // (caller countnext mynext)
        swap
        // (caller mynext countnext)
        exec
        ;
        evens_step;
        swap exec;
        evens_step;
    };

    let mut prog: Sentence = paragraph! { 
        evens;
        drop(1) swap exec;
        drop(1) swap exec;
        halt 
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

        if let Some(next) = control_flow(&mut stack, &mut arena) {
            prog = next;
        } else {
            break;
        }
    }

    println!("{:?}", stack);
    println!("{:?}", arena);
}
