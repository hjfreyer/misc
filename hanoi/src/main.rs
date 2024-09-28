#![allow(unused)]

mod ast;
mod flat;
#[macro_use]
mod macros;

use flat::{
    Builtin, Code, CodeIndex, DeclIndex, Library, Pointer, SentenceIndex, Value, Word, WordIndex,
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
            stack.push(Value::Pointer(closure, code));
        }
        Word::Builtin(Builtin::IsCode) => {
            let value = Value::Bool(match stack.pop().unwrap() {
                Value::Pointer(_, _) => true,
                _ => false,
            });
            stack.push(value)
        }
        Word::PushDecl(decl_index) => {
            stack.push(Value::Pointer(vec![], lib.decls[*decl_index].code))
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
                Some(lib.code_words(true_case))
            } else {
                stack.extend(false_curry);
                Some(lib.code_words(false_case))
            }
        }
        "exec" => {
            let (push, next) = stack.pop().unwrap().into_code(lib).unwrap();
            stack.extend(push);
            Some(lib.code_words(next))
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
            .code_words(lib.decls.last().unwrap().code)
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
    for decl in lib.decls.keys() {
        res.extend(print_decl(lib, decl, styles))
    }
    res
}

fn print_decl(lib: &Library, idx: DeclIndex, styles: &Styles) -> Text<'static> {
    let decl = &lib.decls[idx];
    std::iter::once(Line::raw(format!("let {} = {{", decl.name)))
        .chain(print_code(lib, decl.code, 2, styles, Style::new()).into_iter())
        .chain(std::iter::once("}".into()))
        .collect()
}

fn print_code(
    lib: &Library,
    code_index: CodeIndex,
    indent: usize,
    styles: &Styles,
    mut line_style: Style,
) -> Text<'static> {
    line_style = line_style.patch(styles.codes[code_index]);

    let code = &lib.codes[code_index];
    match code {
        Code::Sentence(styled_sentence) => {
            print_sentence(lib, *styled_sentence, indent, styles, line_style).into()
        }
        Code::AndThen(styled_sentence, styled_code) => {
            let mut line = print_sentence(lib, *styled_sentence, indent, styles, line_style);
            line.push_span(Span::raw(";"));
            std::iter::once(line)
                .chain(print_code(lib, *styled_code, indent, styles, line_style).into_iter())
                .collect()
        }
        Code::If {
            cond,
            true_case,
            false_case,
        } => {
            let mut line = print_sentence(lib, *cond, indent, styles, line_style);
            line.push_span(" if {");
            std::iter::once(line)
                .chain(print_code(lib, *true_case, indent + 2, styles, line_style).into_iter())
                .chain(std::iter::once(
                    Line::raw(format!("{}}} else {{", " ".repeat(indent))).style(line_style),
                ))
                .chain(print_code(lib, *false_case, indent + 2, styles, line_style).into_iter())
                .chain(std::iter::once(
                    Line::raw(format!("{}}}", " ".repeat(indent))).style(line_style),
                ))
                .collect()
        }
    }
}

fn print_sentence(
    lib: &Library,
    sentence_idx: SentenceIndex,
    indent: usize,
    styles: &Styles,
    line_style: Style,
) -> Line<'static> {
    let sentence = &lib.sentences[sentence_idx];
    Line::from_iter(
        std::iter::once(Span::raw(" ".repeat(indent))).chain(Itertools::intersperse(
            sentence.0.iter().map(|w| print_word(lib, *w, styles)),
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
        Word::PushDecl(decl_idx) => lib.decls[*decl_idx].name.clone().into(),
    };
    res.style(styles.words[word_idx])
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Judgement {
    Eq(usize, usize),
    OutExact(usize, Value),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Type {
    pub arity_in: usize,
    pub arity_out: usize,
    pub judgements: Vec<Judgement>,
}

impl Type {
    pub fn pad(&self) -> Self {
        Type {
            arity_in: self.arity_in + 1,
            arity_out: self.arity_out + 1,
            judgements: self
                .judgements
                .iter()
                .cloned()
                .chain(std::iter::once(Judgement::Eq(
                    self.arity_in,
                    self.arity_out,
                )))
                .collect(),
        }
    }
}

fn get_type(lib: &Library, w: Word) -> Type {
    match w {
        Word::Push(value) => Type {
            arity_in: 0,
            arity_out: 1,
            judgements: vec![Judgement::OutExact(0, value.clone())],
        },
        Word::PushDecl(decl_index) => Type {
            arity_in: 0,
            arity_out: 1,
            judgements: vec![Judgement::OutExact(
                0,
                Value::Pointer(vec![], lib.decls[decl_index].code),
            )],
        },
        Word::Copy(idx) => Type {
            arity_in: idx + 1,
            arity_out: idx + 2,
            judgements: (0..(idx + 1))
                .map(|i| Judgement::Eq(i, i + 1))
                .chain(std::iter::once(Judgement::Eq(idx, 0)))
                .collect(),
        },
        Word::Drop(idx) => Type {
            arity_in: idx + 1,
            arity_out: idx,
            judgements: (0..idx).map(|i| Judgement::Eq(i, i)).collect(),
        },
        Word::Move(idx) => Type {
            arity_in: idx + 1,
            arity_out: idx + 1,
            judgements: (0..idx)
                .map(|i| Judgement::Eq(i, i + 1))
                .chain(std::iter::once(Judgement::Eq(idx, 0)))
                .collect(),
        },
        Word::Builtin(builtin) => match builtin {
            Builtin::Add => Type {
                arity_in: 2,
                arity_out: 1,
                judgements: vec![],
            },
            Builtin::Eq => todo!(),
            Builtin::Curry => Type {
                arity_in: 2,
                arity_out: 1,
                judgements: vec![],
            },
            Builtin::Or => todo!(),
            Builtin::And => todo!(),
            Builtin::Not => todo!(),
            Builtin::IsCode => todo!(),
        },
    }
}

fn compose_types(mut t1: Type, mut t2: Type) -> Type {
    while t1.arity_out < t2.arity_in {
        t1 = t1.pad()
    }
    while t2.arity_in < t1.arity_out {
        t2 = t2.pad()
    }

    let mut res: Vec<Judgement> = vec![];
    for j1 in t1.judgements {
        match j1 {
            Judgement::Eq(i1, o1) => {
                for j2 in t2.judgements.iter() {
                    match j2 {
                        Judgement::Eq(i2, o2) => {
                            if o1 == *i2 {
                                res.push(Judgement::Eq(i1, *o2));
                            }
                        }
                        Judgement::OutExact(_, _) => {}
                    }
                }
            }
            Judgement::OutExact(o1, value) => {
                for j2 in t2.judgements.iter() {
                    match j2 {
                        Judgement::Eq(i2, o2) => {
                            if o1 == *i2 {
                                res.push(Judgement::OutExact(*o2, value.clone()));
                            }
                        }
                        Judgement::OutExact(_, _) => {}
                    }
                }
            }
        }
    }

    for j2 in t2.judgements {
        match j2 {
            Judgement::Eq(i2, o2) => {}
            Judgement::OutExact(o, value) => res.push(Judgement::OutExact(o, value)),
        }
    }

    Type {
        arity_in: t1.arity_in,
        arity_out: t2.arity_out,
        judgements: res,
    }
}

fn sentence_type(lib: &Library, sidx: SentenceIndex) -> Type {
    lib.sentence_words(sidx)
        .into_iter()
        .map(|(w, p)| get_type(lib, w))
        .reduce(compose_types)
        .unwrap()
}

fn code_type(lib: &Library, cidx: CodeIndex) -> Type {
    lib.code_words(cidx)
        .into_iter()
        .map(|(w, p)| get_type(lib, w))
        .reduce(compose_types)
        .unwrap()
}

fn ptr_type(lib: &Library, push: &[Value], code: CodeIndex) -> Type {
    if push.is_empty() {
        code_type(lib, code)
    } else {
        let push_type = push
            .iter()
            .map(|v| get_type(lib, Word::Push(v.clone())))
            .reduce(compose_types)
            .unwrap();
        compose_types(push_type, code_type(lib, code))
    }
}

fn multi_code_type(lib: &Library, cidx: CodeIndex) -> Type {
    let mut t = code_type(lib, cidx);

    if !t
        .judgements
        .iter()
        .any(|j| *j == Judgement::OutExact(0, Value::Symbol("exec")))
    {
        return t;
    }

    let Some((next_push, next_code)) = t.judgements.iter().find_map(|j| match j {
        Judgement::OutExact(1, Value::Pointer(push, code)) => Some((push, *code)),
        _ => None,
    }) else {
        return t;
    };

    let next_type = ptr_type(lib, &next_push, next_code);

    vec![
        t,
        get_type(lib, Word::Drop(0)),
        get_type(lib, Word::Drop(0)),
        next_type,
    ]
    .into_iter()
    .reduce(compose_types)
    .unwrap()
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
    let lib: ast::Library = lib! {
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
            // (caller generator self mynext)
            mv(2) *exec;
            // (caller self (iternext X *yield)|(*ok))
            copy(0) *yield eq if {
                // (caller self iternext X *yield)
                drop(0) drop(0) mv(1)
                // (caller iternext self)
                copy(0) *exec
            } else {

            }
        };

        let is_generator = {
            is_generator_rec is_generator_rec *exec
        };

        let true_test = {
            count is_generator *exec;
            *yield eq *assert
        };
    };
    let lib = Library::from_ast(lib);

    let prog = lib
        .code_words(lib.decls.last().unwrap().code)
        .into_iter()
        .rev()
        .collect();
    let debugger = Debugger {
        vm: Vm {
            lib,
            prog,
            stack: vec![],
            arena: Arena { buffers: vec![] },
        },

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

        let mut count_type = multi_code_type(&vm.lib, vm.lib.decls.last().unwrap().code);

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
