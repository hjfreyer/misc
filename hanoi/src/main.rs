#![allow(unused)]

mod ast;
mod debugger;
mod flat;
mod vm;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use flat::{Builtin, Closure, Entry, InnerWord, Library, SentenceIndex, Value};
use itertools::Itertools;
use vm::{EvalError, ProgramCounter, Vm};

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

    let vm = Vm::new(ast.namespace);

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
    type Item = Result<Value, EvalError<'t>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut res = match self.vm.run_to_trap() {
            Ok(res) => res,
            Err(e) => return Some(Err(e)),
        };

        let Some(Value::Symbol(result)) = res.pop() else {
            panic!();
        };

        match result.as_str() {
            "yield" => {
                let item = res.pop().unwrap();
                let Some(Value::Pointer(mut closure)) = res.pop() else {
                    panic!("bad value")
                };
                closure
                    .0
                    .push(Value::Pointer(Closure(vec![], SentenceIndex::TRAP)));
                self.vm.jump_to(closure);

                Some(Ok(item))
            }
            "eos" => None,
            _ => panic!(),
        }
    }
}

fn test(file: PathBuf) -> anyhow::Result<()> {
    let code = std::fs::read_to_string(&file)?;

    let ast = ast::Module::from_str(&code).unwrap();

    let mut vm = Vm::new(ast.namespace);

    let Some(Entry::Namespace(tests_ns)) = vm.lib.namespaces.first().unwrap().get("tests") else {
        panic!("no namespace named tests")
    };
    let Some(Entry::Value(Value::Pointer(enumerate))) =
        vm.lib.namespaces[*tests_ns].get("enumerate")
    else {
        panic!("no procedure named enumerate")
    };
    assert_eq!(enumerate.0, vec![]);
    let enumerate = enumerate.1;

    let Some(Entry::Value(Value::Pointer(run))) = vm.lib.namespaces[*tests_ns].get("run") else {
        panic!("no procedure named run")
    };
    assert_eq!(run.0, vec![]);
    let run = run.1;

    vm.jump_to(Closure(
        vec![Value::Pointer(Closure(vec![], SentenceIndex::TRAP))],
        enumerate,
    ));

    for tc in (IterReader { vm: &mut vm }).collect_vec() {
        let Value::Symbol(tc_name) = tc.unwrap() else {
            panic!()
        };

        println!("Running test: {}", tc_name);

        vm.jump_to(Closure(
            vec![
                Value::Pointer(Closure(vec![], SentenceIndex::TRAP)),
                Value::Symbol(tc_name),
            ],
            run,
        ));

        let mut res = vm.run_to_trap().unwrap();

        let Value::Symbol(result) = res.pop().unwrap() else {
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
