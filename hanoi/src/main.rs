#![allow(unused)]

mod model;
#[macro_use]
mod macros;

use model::*;

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

    let mut prog: Vec<Word> = vec![
        Word::Push(lib.decls.last().unwrap().value.clone()),
        Word::Push(Value::Symbol("exec")),
    ];
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
