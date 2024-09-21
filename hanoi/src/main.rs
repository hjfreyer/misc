#![allow(unused)]

mod model;
#[macro_use]
mod macros;

use itertools::Itertools;
use model::*;
use ratatui::{
    crossterm::event::{self, KeyEventKind},
    layout::{Constraint, Layout},
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

fn eval(lib: &Library, stack: &mut Vec<Value>, w: &Word) {
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
        Word::Push(v) => stack.push(v.clone()),
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
            // let code = stack.pop().unwrap().into_code(lib).unwrap();
            // let Some(val) = stack.pop() else { panic!() };
            // stack.push(Value::Quote(Box::new(Code::Curried(val, Box::new(code)))));
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

fn control_flow(lib: &Library, stack: &mut Vec<Value>, arena: &mut Arena) -> Option<LibPointer> {
    let Some(Value::Symbol(op)) = stack.pop() else {
        panic!("bad value")
    };
    match op {
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
        // "if" => {
        //     let false_case = stack.pop().unwrap().into_code(lib).unwrap();
        //     let true_case = stack.pop().unwrap().into_code(lib).unwrap();
        //     let Some(Value::Bool(cond)) = stack.pop() else {
        //         panic!()
        //     };
        //     if cond {
        //         Some(true_case.into_words())
        //     } else {
        //         Some(false_case.into_words())
        //     }
        // }
        "exec" => {
            let (push, next) = stack.pop().unwrap().into_code(lib).unwrap();
            stack.extend(push);
            Some(next)
        }
        // "halt" => None,
        _ => panic!(),
    }
}

struct Debugger {
    lib: Library,
    pointer: Option<LibPointer>,
    stack: Vec<Value>,
    arena: Arena,
}

impl Debugger {
    fn step(&mut self) {
        eprintln!("step: {:?}", self.pointer);
        if let Some(pointer) = &mut self.pointer {
            let in_decl = &self.lib.decls[pointer.0].value;
            let new_ptr = match (&mut pointer.1, in_decl) {
                (CodePointer::Sentence(idx), Code::Sentence(sentence)) => {
                    let word = &sentence.0[*idx];
                    eval(&self.lib, &mut self.stack, word);
                    *idx += 1;
                    Some(LibPointer(pointer.0, CodePointer::Sentence(*idx + 1)))
                }
                (CodePointer::Sentence(idx), Code::AndThen(sentence, code)) => {
                    if *idx == 0 {
                        self.stack.push(Value::Pointer(
                            vec![],
                            LibPointer(
                                pointer.0,
                                CodePointer::AndThenContinue(Box::new(code.start_pointer())),
                            ),
                        ))
                    }

                    let word = &sentence.0[*idx];
                    eval(&self.lib, &mut self.stack, word);

                    if *idx + 1 == sentence.0.len() {
                        None
                    } else {
                        Some(LibPointer(pointer.0, CodePointer::Sentence(*idx + 1)))
                    }
                }
                (
                    CodePointer::Sentence(_),
                    Code::If {
                        cond,
                        true_case,
                        false_case,
                    },
                ) => todo!(),
                (CodePointer::AndThenContinue(code_pointer), Code::Sentence(sentence)) => todo!(),
                (CodePointer::AndThenContinue(code_pointer), Code::AndThen(sentence, code)) => {
                    todo!()
                }
                (
                    CodePointer::AndThenContinue(code_pointer),
                    Code::If {
                        cond,
                        true_case,
                        false_case,
                    },
                ) => todo!(),
                (CodePointer::Curried(code_pointer), Code::Sentence(sentence)) => todo!(),
                (CodePointer::Curried(code_pointer), Code::AndThen(sentence, code)) => todo!(),
                (
                    CodePointer::Curried(code_pointer),
                    Code::If {
                        cond,
                        true_case,
                        false_case,
                    },
                ) => todo!(),
                (CodePointer::IfTrue(code_pointer), Code::Sentence(sentence)) => todo!(),
                (CodePointer::IfTrue(code_pointer), Code::AndThen(sentence, code)) => todo!(),
                (
                    CodePointer::IfTrue(code_pointer),
                    Code::If {
                        cond,
                        true_case,
                        false_case,
                    },
                ) => todo!(),
                (CodePointer::IfFalse(code_pointer), Code::Sentence(sentence)) => todo!(),
                (CodePointer::IfFalse(code_pointer), Code::AndThen(sentence, code)) => todo!(),
                (
                    CodePointer::IfFalse(code_pointer),
                    Code::If {
                        cond,
                        true_case,
                        false_case,
                    },
                ) => todo!(),
            };
            self.pointer = new_ptr;
        } else {
            self.pointer = control_flow(&self.lib, &mut self.stack, &mut self.arena)
        }
    }

    fn code(&self) -> Paragraph {
        // let mut buf = String::new();
        // let mut printer = PrettyPrinter {
        //     f: &mut buf,
        //     indent: "".to_string(),
        // };
        // printer.print_lib(&self.lib);
        // Paragraph::new(buf).white().on_blue()
        Paragraph::new(print_lib(&self.lib, self.pointer.as_ref()))
            .white()
            .on_blue()
    }

    fn stack(&self) -> List {
        let items: Vec<ListItem> = self
            .stack
            .iter()
            .map(|v| ListItem::new(format!("{:?}", v)))
            .collect();
        List::new(items)
    }

    fn render_program(&self, frame: &mut Frame) {
        let layout = Layout::horizontal(Constraint::from_percentages([50, 50])).split(frame.area());

        frame.render_widget(self.code(), layout[0]);
        frame.render_widget(self.stack(), layout[1]);
    }
}

pub fn print_lib(lib: &Library, ptr: Option<&LibPointer>) -> Text<'static> {
    let mut res = Text::default();
    for (idx, decl) in lib.decls.iter().enumerate() {
        res.extend(print_decl(
            "".to_string(),
            decl,
            if let Some(ptr) = ptr {
                if idx == ptr.0 {
                    Some(&ptr.1)
                } else {
                    None
                }
            } else {
                None
            },
        ))
    }
    res
}

fn print_decl(mut indent: String, decl: &Decl, ptr: Option<&CodePointer>) -> Text<'static> {
    let mut res = Text::raw(format!("{}let {} = {{\n", indent, decl.name));
    indent += "  ";
    res.extend(print_code(indent.clone(), &decl.value, ptr));
    indent.truncate(indent.len() - 2);
    res.extend(Text::raw(format!("{}}};\n\n", indent)));
    res
}

fn print_code(mut indent: String, value: &Code, ptr: Option<&CodePointer>) -> Text<'static> {
    match (value, ptr) {
        (Code::Sentence(sentence), None) => print_sentence(indent, sentence, None).into(),
        (Code::Sentence(sentence), Some(CodePointer::Sentence(idx))) => {
            print_sentence(indent, sentence, Some(*idx)).into()
        }
        (Code::Sentence(_), _) => panic!(),
        (Code::AndThen(sentence, code), None) => {
            let mut res = Text::default();
            res.push_line(print_sentence(indent.clone(), sentence, None));
            res.extend(print_code(indent, code, None));
            res
        }
        (Code::AndThen(sentence, code), Some(CodePointer::Sentence(idx))) => {
            let mut res = Text::default();
            res.push_line(print_sentence(indent.clone(), sentence, Some(*idx)));
            res.extend(print_code(indent, code, None));
            res
        }
        (Code::AndThen(sentence, code), Some(CodePointer::AndThenContinue(code_pointer))) => {
            let mut res = Text::default();
            res.push_line(print_sentence(indent.clone(), sentence, None));
            res.extend(print_code(indent, code, Some(&code_pointer)));
            res
        }
        (Code::AndThen(sentence, code), _) => panic!(),
        (
            Code::If {
                cond,
                true_case,
                false_case,
            },
            None,
        ) => {
            let mut res = Text::raw("");
            res.push_line(print_sentence(indent.clone(), cond, None));
            res.extend(Text::raw("if {"));
            indent += "  ";
            res.extend(print_code(indent.clone(), true_case, None));
            indent.truncate(indent.len() - 2);
            res.extend(Text::raw(format!("{}}} else {{", indent.clone())));
            indent += "  ";
            res.extend(print_code(indent.clone(), false_case, None));
            indent.truncate(indent.len() - 2);
            res.extend(Text::raw(format!("{}}};", indent.clone())));
            res
        }
        (
            Code::If {
                cond,
                true_case,
                false_case,
            },
            Some(CodePointer::Sentence(_)),
        ) => todo!(),
        (
            Code::If {
                cond,
                true_case,
                false_case,
            },
            Some(CodePointer::AndThenContinue(code_pointer)),
        ) => todo!(),
        (
            Code::If {
                cond,
                true_case,
                false_case,
            },
            Some(CodePointer::Curried(code_pointer)),
        ) => todo!(),
        (
            Code::If {
                cond,
                true_case,
                false_case,
            },
            Some(CodePointer::IfTrue(code_pointer)),
        ) => todo!(),
        (
            Code::If {
                cond,
                true_case,
                false_case,
            },
            Some(CodePointer::IfFalse(code_pointer)),
        ) => todo!(),
        // (Code::Sentence(sentence), _) => Text::raw(format!("{}{:?}", indent, sentence)),
        // Code::AndThen(sentence, code) => {
        //     let mut res = Text::raw(format!("{}{:?};", indent, sentence));
        //     res.extend(print_code(indent, code));
        //     res
        // }
        // Code::If {
        //     cond,
        //     true_case,
        //     false_case,
        // } => {
        //     let mut res = Text::raw(format!("{}{:?} if {{", indent, cond));
        //     indent += "  ";
        //     res.extend(print_code(indent.clone(), true_case));
        //     indent.truncate(indent.len() - 2);
        //     res.extend(Text::raw(format!("{}}} else {{", indent.clone())));
        //     indent += "  ";
        //     res.extend(print_code(indent.clone(), false_case));
        //     indent.truncate(indent.len() - 2);
        //     res.extend(Text::raw(format!("{}}};", indent.clone())));
        //     res
        // }
    }
}

fn print_sentence(mut indent: String, value: &Sentence, ptr: Option<usize>) -> Line<'static> {
    let mut res = Line::raw(indent);
    res.extend(Itertools::intersperse(
        value.0.iter().enumerate().map(|(idx, w)| {
            let text = Span::raw(format!("{:?}", w));
            if ptr == Some(idx) {
                text.bold().on_cyan()
            } else {
                text
            }
        }),
        Span::raw(" "),
    ));
    res
}

fn run(mut terminal: DefaultTerminal, mut debugger: Debugger) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| {
            debugger.render_program(frame);
            // frame.render_widget(greeting, frame.area());
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
                return Ok(());
            }

            if key.kind == KeyEventKind::Press && key.code == event::KeyCode::Right {
                debugger.step();
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let lib = lib! {
        let test_malloc = {
            #4 *malloc;
            #12 #1 mv(3) *set_mem;
            *halt
        };

        let count = {
            // (caller next)
            #1 *yield
            // (caller next 1 *yield)
            mv(3)
            // (next 1 caller)
            *exec;
            #2 *yield mv(3) *exec;
            #3 *yield mv(3) *exec;
            *eos mv(1) *exec
        };

        let evens_step = {
            // (caller (countnext 1 *yield)|(*eos) evensnext)
            copy(1) *yield eq if {
                // (caller countnext 1 *yield evensnext)
                mv(2) copy(0) add mv(2)
                // (caller countnext evensnext 2*i *yield)
                mv(3) mv(3)
                // (caller 2*i *yield countnext evensnext)
                curry
                // (caller 2*i *yield evensnext)
                mv(2) mv(2) mv(3)
                // (evensnext 2*i *yield caller)
                *exec
            } else {
                // (caller *eos evensnext)
                drop(1) mv(1) *exec
            }
        };

        let evens = {
            // (caller mynext)
            count *exec;
            // (caller (countnext 1 *yield)|(*eos) mynext)
            evens_step *exec;
            mv(1) *exec;

            // // (caller countnext 1 *yield mynext)
            // evens_step *exec;
            // // (caller countnext mynext)
            // mv(1)
            // // (caller mynext countnext)
            // *exec
            // ;
            // evens_step *exec;
            // mv(1) *exec;
            // evens_step *exec;
        };

        let main = {
            evens *exec;
            drop(1) drop(1) mv(1) *exec;
            drop(1) drop(1) mv(1) *exec;
           *halt
        };
    };

    // println!("{:?}", lib);
    // // let inc = paragraph! { 1 add };

    // // let count: Sentence = paragraph! {
    // //     // (caller next)
    // //     1
    // //     // (caller next 1)
    // //     mv(2)
    // //     // (next 1 caller)
    // //     exec;
    // //     2 mv(2) exec;
    // //     3 mv(2) exec;
    // //     eos
    // // };

    // // let mut prog: Sentence = paragraph! {
    // //     evens;
    // //     drop(1) swap exec;
    // //     drop(1) swap exec;
    // //     halt
    // // };

    // let mut prog: Vec<Word> = vec![
    //     Word::Push(lib.decls.last().unwrap().value.clone()),
    //     Word::Push(Value::Symbol("exec")),
    // ];
    // let mut stack = vec![];
    // let mut arena = Arena { buffers: vec![] };

    // loop {
    //     for w in prog {
    //         println!("{:?}", w);
    //         eval(&lib, &mut stack, w);
    //         println!("{:?}", stack);
    //         println!("");
    //     }

    //     if let Some(next) = control_flow(&lib, &mut stack, &mut arena) {
    //         prog = next;
    //     } else {
    //         break;
    //     }
    // }

    // println!("{:?}", stack);
    // println!("{:?}", arena);

    let pointer = LibPointer(lib.decls.len() - 1, {
        let code = &lib.decls.last().unwrap().value;
        code.start_pointer()
    });
    let debugger = Debugger {
        lib,
        pointer: Some(pointer),
        stack: vec![],
        arena: Arena { buffers: vec![] },
    };

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal, debugger);
    ratatui::restore();
    app_result
}
