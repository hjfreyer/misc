#![allow(unused)]

mod ast;
mod debugger;
mod flat;
mod vm;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use flat::{
    Builtin, Entry, InnerWord, Library, Namespace, NamespaceIndex, SentenceIndex, Value, ValueView,
    Word,
};
use itertools::Itertools;
use ratatui::{
    crossterm::event::{self, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{self, List, ListItem, ListState, Paragraph, Row, ScrollbarState, Table, TableState},
    DefaultTerminal, Frame,
};
use typed_index_collections::TiVec;
use vm::{Arena, Vm};

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

    let Entry::Value(Value::Pointer(_, main)) = lib.root_namespace().get("main").unwrap() else {
        panic!()
    };
    let prog = lib.sentences[*main]
        .words
        .clone()
        .into_iter()
        .rev()
        .collect();

    let mut vm = Vm {
        lib,
        prog,
        stack: vec![],
        arena: Arena { buffers: vec![] },
    };

    let debugger = debugger::Debugger::new(&code, vm);

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = debugger::run(terminal, debugger);
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
                    Value::Pointer(vec![], SentenceIndex::TRAP).into(),
                    resume.into(),
                    Value::Symbol("exec".to_owned()).into(),
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
        Value::Pointer(vec![], SentenceIndex::TRAP).into(),
        Value::Symbol("enumerate".to_string()).into(),
        Value::Symbol("tests".to_string()).into(),
        Value::Namespace(lib.namespaces.first_key().unwrap()).into(),
        InnerWord::Builtin(Builtin::Get).into(),
        InnerWord::Builtin(Builtin::Get).into(),
        Value::Symbol("exec".to_string()).into(),
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
            Value::Pointer(vec![], SentenceIndex::TRAP).into(),
            Value::Symbol(tc_name).into(),
            Value::Symbol("run".to_string()).into(),
            Value::Symbol("tests".to_string()).into(),
            Value::Namespace(vm.lib.namespaces.first_key().unwrap()).into(),
            InnerWord::Builtin(Builtin::Get).into(),
            InnerWord::Builtin(Builtin::Get).into(),
            Value::Symbol("exec".to_string()).into(),
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
