#![allow(unused)]

mod ast;
mod flat;
#[macro_use]
mod macros;

use flat::{
    Builtin, Code, CodeIndex, CodeRef, CodeView, DeclIndex, DeclRef, Library, Pointer,
    SentenceIndex, SentenceRef, Value, Word, WordIndex,
};
use itertools::Itertools;
use ratatui::{
    crossterm::event::{self, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use typed_index_collections::TiVec;

fn eval(lib: &Library, stack: &mut Vec<Value>, w: &Word) {
    match w {
        Word::Builtin(Builtin::Add) => {
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
        Word::Push(v) => stack.push(v.clone()),
        Word::Builtin(Builtin::Eq) => {
            let Some(a) = stack.pop() else {
                panic!("bad value")
            };
            let Some(b) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a == b));
        }
        Word::Builtin(Builtin::Curry) => {
            let (mut closure, code) = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(val) = stack.pop() else { panic!() };
            closure.insert(0, val);
            stack.push(Value::Pointer(closure, code.idx));
        }
        Word::Builtin(Builtin::IsCode) => {
            let value = Value::Bool(match stack.pop().unwrap() {
                Value::Pointer(_, _) => true,
                _ => false,
            });
            stack.push(value)
        }
        Word::Builtin(Builtin::And) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a && b));
        }
        Word::Builtin(Builtin::Or) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            let Some(Value::Bool(b)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(a || b));
        }
        Word::Builtin(Builtin::Not) => {
            let Some(Value::Bool(a)) = stack.pop() else {
                panic!("bad value")
            };
            stack.push(Value::Bool(!a));
        }
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

fn control_flow(
    lib: &Library,
    stack: &mut Vec<Value>,
    arena: &mut Arena,
) -> Option<Vec<(Word, Pointer)>> {
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
        "if" => {
            let (false_curry, false_case) = stack.pop().unwrap().into_code(lib).unwrap();
            let (true_curry, true_case) = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(Value::Bool(cond)) = stack.pop() else {
                panic!()
            };
            if cond {
                stack.extend(true_curry);
                Some(true_case.words())
            } else {
                stack.extend(false_curry);
                Some(false_case.words())
            }
        }
        "exec" => {
            let (push, next) = stack.pop().unwrap().into_code(lib).unwrap();
            stack.extend(push);
            Some(next.words())
        }
        "assert" => None,
        // "halt" => None,
        unk => panic!("unknown symbol: {}", unk),
    }
}

struct Vm {
    lib: Library,
    prog: Vec<(Word, Pointer)>,
    stack: Vec<Value>,
    arena: Arena,
}

impl Vm {
    pub fn new(ast: ast::Library) -> Self {
        let lib = Library::from_ast(ast);

        let prog = lib
            .decls()
            .last()
            .unwrap()
            .code()
            .words()
            .into_iter()
            .rev()
            .collect();
        Vm {
            lib,
            prog,
            stack: vec![],
            arena: Arena { buffers: vec![] },
        }
    }

    pub fn step(&mut self) -> bool {
        if let Some((word, ptr)) = self.prog.pop() {
            eprintln!("word: {:?}", word);
            eval(&self.lib, &mut self.stack, &word);
            true
        } else {
            if let Some(new_prog) = control_flow(&self.lib, &mut self.stack, &mut self.arena) {
                self.prog = new_prog.into_iter().rev().collect();
                true
            } else {
                false
            }
        }
    }
}

struct Debugger {
    vm: Vm,

    stack_state: ListState,
}

impl Debugger {
    fn step(&mut self) -> bool {
        self.vm.step()
    }

    fn code(&self) -> Paragraph {
        let styles = Styles {
            codes: self
                .vm
                .lib
                .codes
                .keys()
                .map(|idx| match self.vm.prog.last() {
                    Some((_, Pointer::Code(cidx))) if *cidx == idx => {
                        Style::new().on_cyan().underlined()
                    }
                    _ => Style::new(),
                })
                .collect(),
            words: self
                .vm
                .lib
                .words
                .keys()
                .map(|idx| match self.vm.prog.last() {
                    Some((_, Pointer::Sentence(sidx, offset)))
                        if self.vm.lib.sentences[*sidx].0[*offset] == idx =>
                    {
                        Style::new().on_cyan()
                    }
                    _ => Style::new(),
                })
                .collect(),
        };
        Paragraph::new(print_lib(&self.vm.lib, &styles))
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
                    let mut s = "".to_string();
                    v.format(&mut s, &self.vm.lib).unwrap();
                    s
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

pub fn print_lib(lib: &Library, styles: &Styles) -> Text<'static> {
    let mut res = Text::default();
    for decl in lib.decls() {
        res.extend(print_decl(decl, styles))
    }
    res
}

fn print_decl(decl: DeclRef, styles: &Styles) -> Text<'static> {
    std::iter::once(Line::raw(format!("let {} = {{", decl.name())))
        .chain(print_code(decl.code(), 2, styles, Style::new()).into_iter())
        .chain(std::iter::once("}".into()))
        .collect()
}

fn print_code(
    code: CodeRef,
    indent: usize,
    styles: &Styles,
    mut line_style: Style,
) -> Text<'static> {
    line_style = line_style.patch(styles.codes[code.idx]);

    match code.view() {
        CodeView::Sentence(styled_sentence) => {
            print_sentence(styled_sentence, indent, styles, line_style).into()
        }
        CodeView::AndThen(styled_sentence, styled_code) => {
            let mut line = print_sentence(styled_sentence, indent, styles, line_style);
            line.push_span(Span::raw(";"));
            std::iter::once(line)
                .chain(print_code(styled_code, indent, styles, line_style).into_iter())
                .collect()
        }
        CodeView::If {
            cond,
            true_case,
            false_case,
        } => {
            let mut line = print_sentence(cond, indent, styles, line_style);
            line.push_span(" if {");
            std::iter::once(line)
                .chain(print_code(true_case, indent + 2, styles, line_style).into_iter())
                .chain(std::iter::once(
                    Line::raw(format!("{}}} else {{", " ".repeat(indent))).style(line_style),
                ))
                .chain(print_code(false_case, indent + 2, styles, line_style).into_iter())
                .chain(std::iter::once(
                    Line::raw(format!("{}}}", " ".repeat(indent))).style(line_style),
                ))
                .collect()
        }
    }
}

fn print_sentence(
    sentence: SentenceRef,
    indent: usize,
    styles: &Styles,
    line_style: Style,
) -> Line<'static> {
    Line::from_iter(
        std::iter::once(Span::raw(" ".repeat(indent))).chain(Itertools::intersperse(
            sentence
                .word_idxes()
                .map(|w| print_word(sentence.lib, w, styles)),
            Span::raw(" "),
        )),
    )
    .style(line_style)
}

fn print_word(lib: &Library, word_idx: WordIndex, styles: &Styles) -> Span<'static> {
    let res: Span<'static> = match &lib.words[word_idx] {
        Word::Builtin(b) => b.name().into(),
        Word::Push(value) => format!("{:?}", value).into(),
        Word::Copy(i) => format!("copy({})", i).into(),
        Word::Drop(i) => format!("drop({})", i).into(),
        Word::Move(i) => format!("mv({})", i).into(),
    };
    res.style(styles.words[word_idx])
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
                debugger.stack_state.select_previous();
            }
            if key.kind == KeyEventKind::Press && key.code == event::KeyCode::Down {
                debugger.stack_state.select_next();
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut vm = Vm::new(lib! {
        let map_rec = {
            // (caller fn iter self next)
            mv(2) *exec;
            // (caller fn self (iternext val *yield)|(*ok))
            copy(0) *ok eq if {
                // (caller fn self *ok)
                drop(1) drop(1) mv(1) *exec
            } else {
                // (caller fn self iternext val *yield)
                copy(0) *yield eq if {
                    // (caller fn self iternext val *yield next)
                    mv(2) copy(5)
                    // (caller fn self iternext *yield next val fn)
                    *exec;
                    // (caller fn self iternext *yield mapped *ok)
                    *ok eq if {
                        // (caller fn self iternext *yield mapped)
                        mv(4) mv(3) mv(4)
                        // (caller *yield mapped fn iternext self)
                        copy(0) curry curry curry
                        // (caller *yield mapped mapnext)
                        mv(1) mv(2) mv(3) *exec
                    } else {
                        *panic
                    }
                } else {
                    *panic
                }
            }
        };

        let map = {
            // (caller fn iter)
            map_rec map_rec *exec
        };

        let count_rec = {
            // (caller self i)
            #1 add
            // (caller self (i+1))
            copy(0)
            // (caller self (i+1) (i+1))
            mv(2)
            // (caller (i+1) (i+1) self)
            mv(2)
            // (caller (i+1) self (i+1))
            copy(1)
            // (caller (i+1) self (i+1) self)
            curry
            // (caller (i+1) self [(i+1)](self))
            curry
            // (caller (i+1) [self, (i+1)](self))
            mv(1)
            // (caller nextiter (i+1))
            *yield
            // (caller nextiter (i+1) *yield)
            mv(3) *exec
        };

        let count = {
            count_rec #0 count_rec *exec
        };

        let double = {
            // (caller n)
            copy(0) add *ok mv(2) *exec
        };

        let main_rec = {
            // (iter self next)
            mv(2) *exec;
            // (self iternext val *yield)
            drop(0) drop(0) mv(1) copy(0)
            // (iternext self self)
            *exec
        };

        let main = {
            double count map curry curry main_rec main_rec *exec
        };
    });

    let debugger = Debugger {
        vm,

        stack_state: ListState::default(),
    };

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal, debugger);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {
    use flat::{Judgement, Type};

    use super::*;

    #[test]
    fn basic_assert() {
        let mut vm = Vm::new(lib! {
            let true_test = { #true *assert };
        });

        while vm.step() {}

        assert_eq!(vm.stack, vec![Value::Bool(true)])
    }

    #[test]
    fn concrete_generator() {
        let mut vm = Vm::new(lib! {
            let count = {
                // (caller next)
                #1 *yield
                // (caller next 1 *yield)
                mv(3)
                // (next 1 caller)
                *exec;
                #2 *yield mv(3) *exec;
                #3 *yield mv(3) *exec;
                *ok mv(1) *exec
            };

            let is_generator_rec = {
                // (caller generator self)
                copy(1) is_code if {
                    // (caller generator self mynext)
                    mv(2) *exec;
                    // (caller self (iternext X *yield)|(*ok))
                    copy(0) *yield eq if {
                        // (caller self iternext X *yield)
                        drop(0) drop(0) mv(1)
                        // (caller iternext self)
                        copy(0) *exec
                    } else {
                        // (caller self *ok)
                        *ok eq drop(1) mv(1) *exec
                    }
                } else {
                    // (caller generator self)
                    drop(0) drop(0) #false *exec
                }
            };

            let is_generator = {
                is_generator_rec is_generator_rec *exec
            };

            let true_test = {
                count is_generator *exec;
                *assert
            };
        });

        while vm.step() {
            println!("{:?}", vm.stack)
        }

        assert_eq!(vm.stack, vec![Value::Bool(true)])
    }

    #[test]
    fn basic_type() {
        let mut vm = Vm::new(lib! {
            let count_rec = {
                // (caller self i)
                #1 add
                // (caller self (i+1))
                copy(0)
                // (caller self (i+1) (i+1))
                mv(2)
                // (caller (i+1) (i+1) self)
                mv(2)
                // (caller (i+1) self (i+1))
                copy(1)
                // (caller (i+1) self (i+1) self)
                curry
                // (caller (i+1) self [(i+1)](self))
                curry
                // (caller (i+1) [self, (i+1)](self))
                mv(1)
                // (caller nextiter (i+1))
                *yield
                // (caller nextiter (i+1) *yield)
                mv(3) *exec
            };

            let count = {
                count_rec #0 count_rec *exec
            };
        });

        let mut count_type = vm.lib.decls().last().unwrap().code().eventual_type();

        assert_eq!(
            count_type,
            Type {
                arity_in: 1,
                arity_out: 5,
                judgements: vec![
                    Judgement::Eq(0, 1),
                    Judgement::OutExact(2, Value::Symbol("yield")),
                    Judgement::OutExact(0, Value::Symbol("exec")),
                ]
            }
        )
    }
}
