#![allow(unused)]

mod ast;
mod flat;

mod vm;

use flat::{
    Builtin, Code, CodeIndex, CodeRef, CodeView, Entry, EntryView, Library, Namespace,
    NamespaceIndex, Pointer, SentenceIndex, SentenceRef, Value, Word, WordIndex,
};
use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;
use ratatui::{
    crossterm::event::{self, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{List, ListItem, ListState, Paragraph, ScrollbarState},
    DefaultTerminal, Frame,
};
use typed_index_collections::TiVec;
use vm::{Arena, Vm};

struct Debugger {
    code: String,
    vm: Vm,

    code_scroll: u16,
    stack_state: ListState,
}

impl Debugger {
    fn step(&mut self) -> bool {
        self.vm.step()
    }

    fn code(&self) -> Paragraph {
        // let styles = Styles {
        //     codes: self
        //         .vm
        //         .lib
        //         .codes
        //         .keys()
        //         .map(|idx| match self.vm.prog.last() {
        //             Some((_, Pointer::Code(cidx))) if *cidx == idx => {
        //                 Style::new().on_cyan().underlined()
        //             }
        //             _ => Style::new(),
        //         })
        //         .collect(),
        //     words: self
        //         .vm
        //         .lib
        //         .words
        //         .keys()
        //         .map(|idx| match self.vm.prog.last() {
        //             Some((_, Pointer::Sentence(sidx, offset)))
        //                 if self.vm.lib.sentences[*sidx].0[*offset] == idx =>
        //             {
        //                 Style::new().on_cyan()
        //             }
        //             _ => Style::new(),
        //         })
        //         .collect(),
        // };
        Paragraph::new(Text::raw(&self.code))
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
    lib.namespaces
        .first()
        .unwrap()
        .0
        .iter()
        .flat_map(|(name, entry)| match entry {
            flat::Entry::Code(code_index) => {
                std::iter::once(Line::raw(format!("let {} = {{", name)))
                    .chain(
                        print_code(
                            CodeRef {
                                lib,
                                idx: *code_index,
                            },
                            2,
                            styles,
                            Style::new(),
                        )
                        .into_iter(),
                    )
                    .chain(std::iter::once("}".into()))
            }
            flat::Entry::Namespace(namespace_index) => todo!(),
        })
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
fn main() -> std::io::Result<()> {
    let (_, path): (String, String) = std::env::args().collect_tuple().expect("specify one path");

    let code = std::fs::read_to_string(&path)?;

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
        code,
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
