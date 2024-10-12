#![allow(unused)]

mod ast;
mod flat;

mod vm;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use flat::{
    Builtin, Code, CodeIndex, CodeRef, CodeView, Entry, EntryView, InnerWord, Library, Namespace,
    NamespaceIndex, SentenceIndex, SentenceRef, Value, ValueView, Word, WordIndex,
};
use itertools::Itertools;
use ratatui::{
    crossterm::event::{self, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{self, List, ListItem, ListState, Paragraph, ScrollbarState},
    DefaultTerminal, Frame,
};
use typed_index_collections::TiVec;
use vm::{Arena, Vm};

struct Debugger<'t> {
    code: &'t str,
    vm: Vm<'t>,

    code_scroll: u16,
    stack_state: ListState,
}

impl<'t> Debugger<'t> {
    fn step(&mut self) -> bool {
        self.vm.step()
    }

    fn code(&self) -> Paragraph {
        let text = if let Some(Word {
            span: Some(span), ..
        }) = self.vm.prog.last()
        {
            let mut res = Text::raw("");
            let mut iter = self.code[..span.start()].lines();
            res.push_span(iter.next().unwrap().on_green());
            while let Some(next) = iter.next() {
                res.push_line(next);
            }
            let mut iter = span.as_str().lines();
            res.push_span(iter.next().unwrap().on_green());
            while let Some(next) = iter.next() {
                res.push_line(next.on_green());
            }
            let mut iter = self.code[span.end()..].lines();
            res.push_span(iter.next().unwrap());
            while let Some(next) = iter.next() {
                res.push_line(next);
            }

            res
        } else {
            Text::raw(self.code)
        };
        Paragraph::new(text)
            .scroll((self.code_scroll, 0))
            .white()
            .on_blue()
    }

    fn stack(&self) -> List<'static> {
        let items: Vec<ListItem> = self
            .vm
            .stack
            .iter()
            .map(|v| {
                ListItem::new({
                    ValueView {
                        lib: &self.vm.lib,
                        value: v,
                    }
                    .to_string()
                })
            })
            .collect();
        List::new(items).highlight_style(Style::new().black().on_white())
    }

    fn render_program(&mut self, frame: &mut Frame) {
        let layout = Layout::horizontal(Constraint::from_percentages([50, 50])).split(frame.area());

        frame.render_widget(self.code(), layout[0]);
        frame.render_stateful_widget(self.stack(), layout[1], &mut self.stack_state);
    }
}

#[derive(Debug, Clone)]
pub struct Styles {
    pub codes: TiVec<CodeIndex, Style>,
    pub words: TiVec<WordIndex, Style>,
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
            if key.kind == KeyEventKind::Press && key.code == event::KeyCode::Up {
                debugger.code_scroll = debugger.code_scroll.saturating_sub(1);
                // debugger.stack_state.select_previous();
            }
            if key.kind == KeyEventKind::Press && key.code == event::KeyCode::Down {
                debugger.code_scroll = debugger.code_scroll.saturating_add(1);
                // debugger.stack_state.select_next();
            }
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Debug { file: PathBuf },
    Test { file: PathBuf },
}

fn debug(file: PathBuf) -> anyhow::Result<()> {
    let code = std::fs::read_to_string(&file)?;

    let ast = ast::Module::from_str(&code).unwrap();

    let lib = Library::from_ast(ast.namespace);

    let EntryView::Code(main) = lib.root_namespace().get("main").unwrap() else {
        panic!()
    };
    let prog = main.words().into_iter().rev().collect();

    let mut vm = Vm {
        lib,
        prog,
        stack: vec![],
        arena: Arena { buffers: vec![] },
    };

    let debugger = Debugger {
        code_scroll: 0,
        code: &code,
        vm,
        stack_state: ListState::default(),
    };

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal, debugger);
    ratatui::restore();
    app_result?;
    Ok(())
}

struct IterReader<'a, 't> {
    vm: &'a mut Vm<'t>,
}

impl<'a, 't> Iterator for IterReader<'a, 't> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        while self.vm.step() {}

        let Value::Symbol(result) = self.vm.stack.pop().unwrap() else {
            panic!();
        };

        match result.as_str() {
            "yield" => {
                let resume = self.vm.stack.pop().unwrap();

                let item = self.vm.stack.pop().unwrap();

                self.vm.prog = vec![
                    Word {
                        inner: InnerWord::Push(Value::Pointer(vec![], CodeIndex::TRAP)),
                        span: None,
                    },
                    Word {
                        inner: InnerWord::Push(resume),
                        span: None,
                    },
                    Word {
                        inner: InnerWord::Push(Value::Symbol("exec".to_owned())),
                        span: None,
                    },
                ];

                self.vm.prog.reverse();

                Some(item)
            }
            "eos" => None,
            _ => panic!(),
        }
    }
}

fn test(file: PathBuf) -> anyhow::Result<()> {
    let code = std::fs::read_to_string(&file)?;

    let ast = ast::Module::from_str(&code).unwrap();

    let lib = Library::from_ast(ast.namespace);

    let mut prog = vec![
        Word {
            inner: InnerWord::Push(Value::Pointer(vec![], CodeIndex::TRAP)),
            span: None,
        },
        Word {
            inner: InnerWord::Push(Value::Symbol("enumerate".to_string())),
            span: None,
        },
        Word {
            inner: InnerWord::Push(Value::Symbol("tests".to_string())),
            span: None,
        },
        Word {
            inner: InnerWord::Push(Value::Namespace(lib.root_namespace().idx)),
            span: None,
        },
        Word {
            inner: InnerWord::Builtin(Builtin::Get),
            span: None,
        },
        Word {
            inner: InnerWord::Builtin(Builtin::Get),
            span: None,
        },
        Word {
            inner: InnerWord::Push(Value::Symbol("exec".to_string())),
            span: None,
        },
    ];
    prog.reverse();

    let mut vm = Vm {
        lib,
        prog,
        stack: vec![],
        arena: Arena { buffers: vec![] },
    };

    for tc in (IterReader { vm: &mut vm }).collect_vec() {
        let Value::Symbol(tc_name) = tc else { panic!() };

        println!("Running test: {}", tc_name);

        vm.prog = vec![
            Word {
                inner: InnerWord::Push(Value::Pointer(vec![], CodeIndex::TRAP)),
                span: None,
            },
            Word {
                inner: InnerWord::Push(Value::Symbol(tc_name)),
                span: None,
            },
            Word {
                inner: InnerWord::Push(Value::Symbol("run".to_string())),
                span: None,
            },
            Word {
                inner: InnerWord::Push(Value::Symbol("tests".to_string())),
                span: None,
            },
            Word {
                inner: InnerWord::Push(Value::Namespace(vm.lib.root_namespace().idx)),
                span: None,
            },
            Word {
                inner: InnerWord::Builtin(Builtin::Get),
                span: None,
            },
            Word {
                inner: InnerWord::Builtin(Builtin::Get),
                span: None,
            },
            Word {
                inner: InnerWord::Push(Value::Symbol("exec".to_string())),
                span: None,
            },
        ];
        vm.prog.reverse();
        while vm.step() {}

        let Value::Symbol(result) = vm.stack.pop().unwrap() else {
            panic!()
        };

        match result.as_str() {
            "pass" => println!("PASS!"),
            "fail" => println!("FAIL!"),
            _ => panic!(),
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Debug { file } => debug(file),
        Commands::Test { file } => test(file),
    }
}

#[cfg(test)]
mod tests {
    use flat::{Judgement, Type};

    use super::*;

    // #[test]
    // fn basic_assert() {
    //     let mut vm = Vm::new(lib! {
    //         let true_test = { #true *assert };
    //     });

    //     while vm.step() {}

    //     assert_eq!(vm.stack, vec![Value::Bool(true)])
    // }

    // #[test]
    // fn concrete_generator() {
    //     let mut vm = Vm::new(lib! {
    //         let count = {
    //             // (caller next)
    //             #1 *yield
    //             // (caller next 1 *yield)
    //             mv(3)
    //             // (next 1 caller)
    //             *exec;
    //             #2 *yield mv(3) *exec;
    //             #3 *yield mv(3) *exec;
    //             *ok mv(1) *exec
    //         };

    //         let is_generator_rec = {
    //             // (caller generator self)
    //             copy(1) is_code if {
    //                 // (caller generator self mynext)
    //                 mv(2) *exec;
    //                 // (caller self (iternext X *yield)|(*ok))
    //                 copy(0) *yield eq if {
    //                     // (caller self iternext X *yield)
    //                     drop(0) drop(0) mv(1)
    //                     // (caller iternext self)
    //                     copy(0) *exec
    //                 } else {
    //                     // (caller self *ok)
    //                     *ok eq drop(1) mv(1) *exec
    //                 }
    //             } else {
    //                 // (caller generator self)
    //                 drop(0) drop(0) #false *exec
    //             }
    //         };

    //         let is_generator = {
    //             is_generator_rec is_generator_rec *exec
    //         };

    //         let true_test = {
    //             count is_generator *exec;
    //             *assert
    //         };
    //     });

    //     while vm.step() {
    //         // println!("{:?}", vm.stack)
    //     }

    //     assert_eq!(vm.stack, vec![Value::Bool(true)])
    // }

    // #[test]
    // fn basic_type() {
    //     let mut vm = Vm::new(lib! {
    //         let count_rec = {
    //             // (caller self i)
    //             #1 add
    //             // (caller self (i+1))
    //             copy(0)
    //             // (caller self (i+1) (i+1))
    //             mv(2)
    //             // (caller (i+1) (i+1) self)
    //             mv(2)
    //             // (caller (i+1) self (i+1))
    //             copy(1)
    //             // (caller (i+1) self (i+1) self)
    //             curry
    //             // (caller (i+1) self [(i+1)](self))
    //             curry
    //             // (caller (i+1) [self, (i+1)](self))
    //             mv(1)
    //             // (caller nextiter (i+1))
    //             *yield
    //             // (caller nextiter (i+1) *yield)
    //             mv(3) *exec
    //         };

    //         let count = {
    //             count_rec #0 count_rec *exec
    //         };
    //     });

    //     let mut count_type = vm.lib.decls().last().unwrap().code().eventual_type();

    //     assert_eq!(
    //         count_type,
    //         Type {
    //             arity_in: 1,
    //             arity_out: 5,
    //             judgements: vec![
    //                 Judgement::Eq(0, 1),
    //                 Judgement::OutExact(2, Value::Symbol("yield")),
    //                 Judgement::OutExact(0, Value::Symbol("exec")),
    //             ]
    //         }
    //     )
    // }
}
