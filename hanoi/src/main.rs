#![allow(unused)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct Library {
    decls: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Decl {
    name: String,
    value: Value,
}

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
    Quote(Box<Code>),
    Handle(usize),
    Bool(bool),
    Reference(String),
}

impl Value {
    fn into_code(self, lib: &Library) -> Option<Code> {
        match self {
            Value::Quote(code) => Some(*code),
            Value::Reference(name) => Some(
                lib.decls
                    .iter()
                    .find_map(|d| {
                        if d.name == name {
                            let Value::Quote(code) = d.value.clone() else {
                                panic!()
                            };
                            Some(*code)
                        } else {
                            None
                        }
                    })
                    .unwrap(),
            ),
            Value::Symbol(_)
            | Value::Usize(_)
            | Value::List(_)
            | Value::Bool(_)
            | Value::Handle(_) => None,
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference(arg0) => write!(f, "{}", arg0),
            Self::Symbol(arg0) => write!(f, "*{}", arg0),
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

fn eval(lib: &Library, stack: &mut Vec<Value>, w: Word) {
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
            let code = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(val) = stack.pop() else { panic!() };
            stack.push(Value::Quote(Box::new(Code::Curried(val, Box::new(code)))));
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
    (add) => {
        Word::Add
    };
    (curry) => {
        Word::Curry
    };
    (copy($idx:expr)) => {
        Word::Copy($idx)
    };
    (drop($idx:expr)) => {
        Word::Drop($idx)
    };
    (mv($idx:expr)) => {
        Word::Move($idx)
    };
    (* $name:ident) => {
        Word::Push(Value::Symbol(stringify!($name)))
    };
    (# $val:expr) => {
        Word::Push(Value::from($val))
    };
    ($name:ident) => {
        Word::Push(Value::Reference(stringify!($name).to_string()))
    };
}

macro_rules! value {
    (@phrasecat ($($phrase:tt)*) ($($tail:tt)*) ) => {
        {
            let mut res :Sentence= Sentence(vec![]);
            res.push(phrase!($($phrase)*));
            res.push(value!(@sent ($($tail)*)));
            res
        }
    };
    (@sent ()) => { Sentence(vec![]) };

    (@sent (* $symbol:ident $($tail:tt)*)) => {
        value!(@phrasecat (* $symbol) ($($tail)*))
    };
    (@sent (# $val:tt $($tail:tt)*)) => {
        value!(@phrasecat (# $val) ($($tail)*))
    };
    (@sent ($flike:ident($($head:tt)*) $($tail:tt)*)) => {
        value!(@phrasecat ($flike($($head)*)) ($($tail)*))
    };
    (@sent ($head:tt $($tail:tt)*)) => {
        value!(@phrasecat ($head) ($($tail)*))
    };
    (@code ($($a:tt)*) ()) => {
        Code::Sentence(
            value!(@sent ($($a)*)),
        )
    };
    (@code ($($a:tt)*) (; $($tail:tt)*)) => {
        Code::AndThen(
            value!(@sent ($($a)*)),
            Box::new(value!(@code () ($($tail)*)))
        )
    };
    (@code ($($a:tt)*) ($head:tt $($tail:tt)*)) => {
        value!(@code ($($a)* $head) ($($tail)*))
    };
    ($i:ident) => {
        Value::Reference(stringify!($i))
    };
    ({$($code:tt)*}) => {
        Value::Quote(Box::new(value!(@code () ($($code)*))))
    };
    ($e:expr) => {
        Value::from($e)
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

#[derive(Clone, PartialEq, Eq)]
enum Code {
    Sentence(Sentence),
    AndThen(Sentence, Box<Code>),
    Curried(Value, Box<Code>),
}

impl Code {
    fn into_words(self) -> Vec<Word> {
        match self {
            Code::Sentence(sentence) => sentence.0,
            Code::AndThen(sentence, code) => {
                let mut res = vec![Word::Push(Value::Quote(code))];
                res.extend(sentence.0);
                res
            }
            Code::Curried(value, code) => {
                let mut res = vec![Word::Push(value)];
                res.extend(code.into_words());
                res
            }
        }
    }
}

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentence(arg0) => arg0.fmt(f),
            Self::AndThen(arg0, arg1) => write!(f, "{:?}; {:?}", arg0, arg1),
            Self::Curried(arg0, arg1) => write!(f, "[{:?}]({:?})", arg0, arg1),
        }
    }
}

macro_rules! lib {
    (@lib () ()) => {
        Library {
            decls: vec![],
        }
    };
    (@lib (let $name:ident = $val:tt;) ($($tail:tt)*)) => {
        {
            let mut lib = lib!($($tail)*);
            lib.decls.insert(0, Decl {
                name: stringify!($name).to_string(),
                value: value!($val),
            });
            lib
        }
    };
    (@lib ($($a:tt)*) ($head:tt $($tail:tt)*)) => {
        lib!(@lib ($($a)* $head) ($($tail)*))
    };
    ($($tail:tt)*) => {
        lib!(@lib () ($($tail)*))
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

fn control_flow(lib: &Library, stack: &mut Vec<Value>, arena: &mut Arena) -> Option<Vec<Word>> {
    let Some(Value::Symbol(op)) = stack.pop() else {
        panic!("bad value")
    };
    match op {
        "malloc" => {
            let Some(Value::Usize(size)) = stack.pop() else {
                panic!()
            };
            let next = stack.pop().unwrap().into_code(lib).unwrap();

            let handle = Value::Handle(arena.buffers.len());

            arena.buffers.push(Buffer { mem: vec![0; size] });

            stack.push(handle);
            Some(next.into_words())
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
            let next = stack.pop().unwrap().into_code(lib).unwrap();

            let buf = arena.buffers.get_mut(handle).unwrap();
            buf.mem[offset] = value;

            Some(next.into_words())
        }
        "get_mem" => {
            let Some(Value::Handle(handle)) = stack.pop() else {
                panic!()
            };
            let Some(Value::Usize(offset)) = stack.pop() else {
                panic!()
            };
            let next = stack.pop().unwrap().into_code(lib).unwrap();

            let buf = arena.buffers.get_mut(handle).unwrap();
            stack.push(Value::Usize(buf.mem[offset]));

            Some(next.into_words())
        }
        "if" => {
            let false_case = stack.pop().unwrap().into_code(lib).unwrap();
            let true_case = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(Value::Bool(cond)) = stack.pop() else {
                panic!()
            };
            if cond {
                Some(true_case.into_words())
            } else {
                Some(false_case.into_words())
            }
        }
        "exec" => {
            let next = stack.pop().unwrap().into_code(lib).unwrap();
            Some(next.into_words())
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

    let lib = lib! {
        let test_malloc = {
            #4 *malloc;
            #12 #1 mv(3) *set_mem;
            *halt
        };

        let count = {
            // (caller next)
            #1
            // (caller next 1)
            mv(2)
            // (next 1 caller)
            *exec;
            #2 mv(2) *exec;
            #3 mv(2) *exec;
            *eos
        };

        let evens_step = {
            // (caller countnext i mynext)
            mv(1) copy(0) add
            // (caller countnext mynext 2*i)
            mv(2) mv(2)
            // (caller 2*i countnext mynext)
            curry
            // (caller 2*i evensnext)
            mv(1) mv(2)
            // (evensnext 2*i caller)
            *exec
        };

        let evens = {
            // (caller mynext)
            count *exec;
            // (caller countnext 1 mynext)
            evens_step *exec;
            // (caller countnext mynext)
            mv(1)
            // (caller mynext countnext)
            *exec
            ;
            evens_step *exec;
            mv(1) *exec;
            evens_step *exec;
        };

        let main = {
            evens *exec;
            drop(1) mv(1) *exec;
            drop(1) mv(1) *exec;
           *halt
        };
    };

    println!("{:?}", lib);
    // let inc = paragraph! { 1 add };

    // let count: Sentence = paragraph! {
    //     // (caller next)
    //     1
    //     // (caller next 1)
    //     mv(2)
    //     // (next 1 caller)
    //     exec;
    //     2 mv(2) exec;
    //     3 mv(2) exec;
    //     eos
    // };

    // let mut prog: Sentence = paragraph! {
    //     evens;
    //     drop(1) swap exec;
    //     drop(1) swap exec;
    //     halt
    // };

    let mut prog: Vec<Word> =
        sentcat![(Word::Push(lib.decls.last().unwrap().value.clone())), exec,].0;
    let mut stack = vec![];
    let mut arena = Arena { buffers: vec![] };

    loop {
        for w in prog {
            println!("{:?}", w);
            eval(&lib, &mut stack, w);
            println!("{:?}", stack);
            println!("");
        }

        if let Some(next) = control_flow(&lib, &mut stack, &mut arena) {
            prog = next;
        } else {
            break;
        }
    }

    println!("{:?}", stack);
    println!("{:?}", arena);
}
