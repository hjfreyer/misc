#![allow(unused)]

#[derive(Debug, Clone)]
enum Word {
    Add,
    Push(Value),
    Cons,
    Snoc,
    Copy(usize),
    Drop(usize),
}

#[derive(Debug, Clone)]
enum Value {
    Symbol(&'static str),
    Usize(usize),
    List(Vec<Value>),
    Quote(Box<Sentence>),
    Handle(usize),
}

#[derive(Debug, Clone)]
enum Sentence {
    Word(Word),
    Sentence(Vec<Sentence>),
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
    }
}

fn flatten(sentence: Sentence) -> Vec<Word> {
    match sentence {
        Sentence::Word(w) => vec![w],
        Sentence::Sentence(s) => s.into_iter().flat_map(|s| flatten(s).into_iter()).collect(),
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
        Self::Word(value.into())
    }
}

impl<I> FromIterator<I> for Sentence
where
    I: Into<Sentence>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        Sentence::Sentence(iter.into_iter().map(|x| x.into()).collect())
    }
}

macro_rules! phrase {
    (copy($idx:expr)) => {
        Sentence::Word(Word::Copy($idx))
    };
    (drop($idx:expr)) => {
        Sentence::Word(Word::Drop($idx))
    };
    ($name:ident) => {
        $name.clone().into()
    };
    ($name:expr) => {
        $name.clone().into()
    };
}

macro_rules! sentence {
    () => {
        Sentence::Sentence(vec![])
    };
    ({$($quote:tt)*} $($tail:tt)*) => {
        Sentence::Sentence(vec![Sentence::Word(Word::Push(Value::Quote(Box::new(sentence!($($quote)*))))), sentence!($($tail)*)])
    };
    ($flike:ident($($head:tt)*) $($tail:tt)*) => {
        Sentence::Sentence(vec![phrase!($flike($($head)*)), sentence!($($tail)*)])
    };
    ($head:tt $($tail:tt)*) => {
        Sentence::Sentence(vec![phrase!($head), sentence!($($tail)*)])
    };
}

macro_rules! paragraph {
    () => {
        Sentence::Sentence(vec![])
    };
    (@accum () -> ($($a:tt)*)) => {
        sentence!($($a)*)
    };
    (@accum (; $($tail:tt)*) -> ($($a:tt)*)) => {
        Sentence::Sentence(vec![Sentence::Word(Word::Push(Value::Quote(Box::new(paragraph!($($tail)*))))), sentence!($($a)*)])
    }; 
    (@accum ($head:tt $($tail:tt)*) -> ($($a:tt)*)) => {
        paragraph!(@accum ($($tail)*) -> ($($a)* $head))
    };
    ($($tail:tt)*) => {
        paragraph!(@accum ($($tail)*) -> () )
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
    let add = Sentence::Word(Word::Add);
    let cons = Sentence::Word(Word::Cons);
    let snoc = Sentence::Word(Word::Snoc);

    let halt = Sentence::Word(Word::Push(Value::Symbol("halt")));
    let malloc = Sentence::Word(Word::Push(Value::Symbol("malloc")));
    let get_mem = Sentence::Word(Word::Push(Value::Symbol("get_mem")));
    let set_mem = Sentence::Word(Word::Push(Value::Symbol("set_mem")));

    let double = sentence!(copy(0) add);

    let swap = sentence!(copy(1) drop(2));

    let mut prog = sentence!({{{double {halt} copy(1) 2 copy(4) set_mem} 1 copy(2) get_mem} 12 1 copy(3) set_mem} 4 malloc);

    let mut prog: Sentence = paragraph!{
        4 malloc;
        12 1 copy(3) set_mem;
        1 copy(2) get_mem;
        copy(1) drop(2) double 2 copy(3) set_mem;
        halt
    };
    let mut stack = vec![];
    let mut arena = Arena { buffers: vec![] };

    loop {
        for w in flatten(prog) {
            println!("{:?}", w);
            eval(&mut stack, w);
            println!("{:?}", stack);
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
            Value::Symbol(s) if s == "halt" => break,
            _ => panic!(),
        }
    }

    println!("{:?}", stack);
    println!("{:?}", arena);
}
