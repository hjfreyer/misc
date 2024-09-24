#![allow(unused)]

mod ast;
mod flat;
#[macro_use]
mod macros;

use flat::{Code, CodeIndex, DeclIndex, Library, Pointer, SentenceIndex, Value, Word, WordIndex};
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
            let (mut closure, code) = stack.pop().unwrap().into_code(lib).unwrap();
            let Some(val) = stack.pop() else { panic!() };
            closure.insert(0, val);
            stack.push(Value::Pointer(closure, code));
        }
        Word::PushDecl(decl_index) => {
            stack.push(Value::Pointer(vec![], lib.decls[*decl_index].code))
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

fn control_flow(lib: &Library, stack: &mut Vec<Value>, arena: &mut Arena) -> Vec<(Word, Pointer)> {
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
                lib.code_words(true_case)
            } else {
                stack.extend(false_curry);
                lib.code_words(false_case)
            }
        }
        "exec" => {
            let (push, next) = stack.pop().unwrap().into_code(lib).unwrap();
            stack.extend(push);
            lib.code_words(next)
        }
        // "halt" => None,
        unk => panic!("unknown symbol: {}", unk),
    }
}

struct Debugger {
    lib: Library,
    prog: Vec<(Word, Pointer)>,
    stack: Vec<Value>,
    arena: Arena,

    stack_state: ListState,
}

impl Debugger {
    fn step(&mut self) {
        if let Some((word, ptr)) = self.prog.pop() {
            eprintln!("word: {:?}", word);
            eval(&self.lib, &mut self.stack, &word);
        } else {
            self.prog = control_flow(&self.lib, &mut self.stack, &mut self.arena)
                .into_iter()
                .rev()
                .collect()
        }
    }

    fn code(&self) -> Paragraph {
        let styles = Styles {
            codes: self
                .lib
                .codes
                .keys()
                .map(|idx| match self.prog.last() {
                    Some((_, Pointer::Code(cidx))) if *cidx == idx => {
                        Style::new().on_cyan().underlined()
                    }
                    _ => Style::new(),
                })
                .collect(),
            words: self
                .lib
                .words
                .keys()
                .map(|idx| match self.prog.last() {
                    Some((_, Pointer::Sentence(sidx, offset)))
                        if self.lib.sentences[*sidx].0[*offset] == idx =>
                    {
                        Style::new().on_cyan()
                    }
                    _ => Style::new(),
                })
                .collect(),
        };
        Paragraph::new(print_lib(&self.lib, &styles))
            .white()
            .on_blue()
    }

    fn stack(&self) -> List<'static> {
        let items: Vec<ListItem> = self
            .stack
            .iter()
            .map(|v| {
                ListItem::new({
                    let mut s = "".to_string();
                    v.format(&mut s, &self.lib).unwrap();
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
                .chain(std::iter::once("} else {".into()))
                .chain(print_code(lib, *false_case, indent + 2, styles, line_style).into_iter())
                .chain(std::iter::once("}".into()))
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
        Word::Add => "add".into(),
        Word::Push(value) => format!("{:?}", value).into(),
        Word::Cons => todo!(),
        Word::Snoc => todo!(),
        Word::Eq => "eq".into(),
        Word::Copy(i) => format!("copy({})", i).into(),
        Word::Drop(i) => format!("drop({})", i).into(),
        Word::Move(i) => format!("mv({})", i).into(),
        Word::Swap => "swap".into(),
        Word::Curry => "curry".into(),
        Word::PushDecl(decl_idx) => lib.decls[*decl_idx].name.clone().into(),
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
            *eos mv(1) *exec
        };

        let double_inner = {
            // (caller iternext self mynext)
            mv(2) *exec;
            // (caller self (iternext 1 *yield)|(*eos))
            *yield eq if {
                // (caller self iternext 1)
                copy(0) add
                // (caller self iternext 2)

                mv(1) mv(2) copy(0) curry curry mv(1) *yield
                // (caller selfnext 1 *yield)
                mv(3) *exec

            } else {
                drop(0) *eos mv(1) *exec
            }
        };

        let double = {
            double_inner double_inner *exec
        };

        let main = {
            count double *exec;
            // (iternext 2 *yield mynext)
            drop(1) drop(1) mv(1) *exec;
            drop(1) drop(1) mv(1) *exec;
            drop(1) drop(1) mv(1) *exec;
            drop(1) drop(1) mv(1) *exec;
           *halt
        };
    };
    let lib = Library::from_ast(lib);

    let prog = lib
        .code_words(lib.decls.last().unwrap().code)
        .into_iter()
        .rev()
        .collect();
    let debugger = Debugger {
        lib,
        prog,
        stack: vec![],
        arena: Arena { buffers: vec![] },

        stack_state: ListState::default(),
    };

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal, debugger);
    ratatui::restore();
    app_result
}
