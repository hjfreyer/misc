use itertools::Itertools;
use ratatui::{
    crossterm::event,
    layout::{Constraint, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{Paragraph, Row, Table, TableState},
    DefaultTerminal,
};

use crate::{
    flat::{ValueView, Word},
    vm::{EvalError, StepResult, Vm},
};

pub struct Debugger<'t> {
    code: &'t str,
    vm: Vm<'t>,

    code_scroll: u16,
    stack_state: TableState,
    error: Option<EvalError<'t>>,
}

impl<'t> Debugger<'t> {
    fn step(&mut self) -> Result<(), EvalError<'t>> {
        if self.error.is_some() {
            return Ok(());
        }

        match self.vm.step() {
            Ok(step) => {
                if let StepResult::Continue = step {
                    if let Some(word) = self.vm.current_word() {
                        if let Some(span) = &word.span {
                            let (line, _) = span.start_pos().line_col();
                            self.code_scroll = (line as u16).saturating_sub(10);
                        }
                    }
                }
            }
            Err(err) => {
                self.error = Some(err);
            }
        }
        Ok(())
    }

    fn code(&self) -> Paragraph {
        let text = if let Some(Word {
            span: Some(span), ..
        }) = self.vm.current_word()
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

    fn stack(&self) -> Table<'static> {
        let names = self
            .vm
            .current_word()
            .and_then(|w| w.names.clone())
            .unwrap_or_else(|| self.vm.stack.iter().map(|_| None).collect());

        let names_width = names
            .iter()
            .filter_map(|n| n.as_ref().map(|s| s.len() + 3))
            .max()
            .unwrap_or_default();

        let mut items: Vec<Row> = self
            .vm
            .stack
            .iter()
            .rev()
            .zip_longest(names.iter())
            .map(|v| {
                let (v, name) = match v {
                    itertools::EitherOrBoth::Both(v, name) => (v, name.as_ref()),
                    itertools::EitherOrBoth::Left(v) => (v, None),
                    itertools::EitherOrBoth::Right(_) => panic!(
                        "name with no value?\nnames: {:?}\n stack: {:?}",
                        names, self.vm.stack.iter().rev().collect_vec()
                    ),
                };
                Row::new([
                    if let Some(name) = name {
                        format!("{} = ", name)
                    } else {
                        "".to_owned()
                    },
                    ValueView {
                        lib: &self.vm.lib,
                        value: v,
                    }
                    .to_string(),
                ])
            })
            .collect();
        items.reverse();
        Table::new(
            items,
            [Constraint::Length(names_width as u16), Constraint::Fill(1)],
        )
        .column_spacing(0)
        .highlight_style(Style::new().black().on_white())
    }

    fn error_text(&self) -> Text {
        let Some(err) = &self.error else {
            return Text::default();
        };
        Text::raw(err.to_string()).red()
    }

    fn render_program(&mut self, frame: &mut ratatui::Frame) {
        let layout = Layout::horizontal(Constraint::from_percentages([50, 50])).split(frame.area());

        let err_text = self.error_text();
        let stack_layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Min(err_text.height() as u16),
        ])
        .split(layout[1]);

        frame.render_widget(self.code(), layout[0]);
        frame.render_widget(err_text, stack_layout[1]);
        frame.render_stateful_widget(self.stack(), stack_layout[0], &mut self.stack_state);
    }

    pub fn new(code: &'t str, vm: crate::vm::Vm<'t>) -> Self {
        Self {
            code_scroll: 0,
            code: &code,
            vm,
            stack_state: TableState::default(),
            error: None,
        }
    }
}

pub fn run(mut terminal: DefaultTerminal, mut debugger: Debugger) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| {
            debugger.render_program(frame);
            // frame.render_widget(greeting, frame.area());
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
                return Ok(());
            }

            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Right {
                debugger.step().unwrap();
            }
            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Up {
                debugger.code_scroll = debugger.code_scroll.saturating_sub(1);
                // debugger.stack_state.select_previous();
            }
            if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Down {
                debugger.code_scroll = debugger.code_scroll.saturating_add(1);
                // debugger.stack_state.select_next();
            }
        }
    }
}
