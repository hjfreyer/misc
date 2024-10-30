use std::{collections::VecDeque, usize};

use derive_more::derive::{From, Into};
use itertools::Itertools;
use pest::Span;
use typed_index_collections::TiVec;

use crate::ast::{self, Expression, InnerExpression};

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SentenceIndex(usize);

impl SentenceIndex {
    pub const TRAP: Self = SentenceIndex(usize::MAX);
}

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct NamespaceIndex(usize);

#[derive(Debug, Clone, Default)]
pub struct Library<'t> {
    pub namespaces: TiVec<NamespaceIndex, Namespace>,
    pub sentences: TiVec<SentenceIndex, Sentence<'t>>,
}

#[derive(Debug, Clone, Default)]
pub struct Namespace(pub Vec<(String, Entry)>);

impl Namespace {
    pub fn get(&self, name: &str) -> Option<&Entry> {
        self.0
            .iter()
            .find_map(|(k, v)| if k == name { Some(v) } else { None })
    }
}

#[derive(Debug, Clone)]
pub enum Entry {
    Value(Value),
    Namespace(NamespaceIndex),
}

impl<'t> Library<'t> {
    pub fn from_ast(lib: ast::Namespace<'t>) -> Self {
        let mut res = Self::default();
        res.visit_ns(lib);
        res
    }

    pub fn root_namespace(&self) -> &Namespace {
        self.namespaces.first().unwrap()
    }

    fn visit_ns(&mut self, ns: ast::Namespace<'t>) -> NamespaceIndex {
        let ns_idx = self.namespaces.push_and_get_key(Namespace::default());

        for decl in ns.decls {
            match decl.value {
                ast::DeclValue::Namespace(namespace) => {
                    let subns = self.visit_ns(namespace);
                    self.namespaces[ns_idx]
                        .0
                        .push((decl.name, Entry::Namespace(subns)));
                }
                ast::DeclValue::Code(code) => {
                    let sentence_idx = self.visit_code(&decl.name, ns_idx, VecDeque::new(), code);
                    self.namespaces[ns_idx].0.push((
                        decl.name,
                        Entry::Value(Value::Pointer(vec![], sentence_idx)),
                    ));
                }
            }
        }
        ns_idx
    }

    fn visit_code(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        names: VecDeque<Option<String>>,
        code: ast::Code<'t>,
    ) -> SentenceIndex {
        match code {
            ast::Code::Sentence(sentence) => self.visit_sentence(name, ns_idx, names, sentence),
            ast::Code::AndThen(sentence, code) => {
                let init = self.convert_sentence(name, ns_idx, names, sentence);
                let and_then = self.visit_code(name, ns_idx, VecDeque::new(), *code);

                self.sentences.push_and_get_key(Sentence {
                    name: init.name,
                    words: std::iter::once(
                        InnerWord::Push(Value::Pointer(vec![], and_then)).into(),
                    )
                    .chain(init.words.into_iter())
                    .collect(),
                })
            }
            ast::Code::If {
                cond,
                true_case,
                false_case,
            } => {
                let cond = self.convert_sentence(name, ns_idx, names, cond);
                let true_case = self.visit_code(name, ns_idx, VecDeque::new(), *true_case);
                let false_case = self.visit_code(name, ns_idx, VecDeque::new(), *false_case);

                self.sentences.push_and_get_key(Sentence {
                    name: cond.name,
                    words: cond
                        .words
                        .into_iter()
                        .chain([
                            Value::Pointer(vec![], true_case).into(),
                            Value::Pointer(vec![], false_case).into(),
                            Value::Symbol("if".to_owned()).into(),
                        ])
                        .collect(),
                })
            }
            ast::Code::Bind {
                name: var_name,
                inner,
                span,
            } => self.visit_code(
                name,
                ns_idx,
                [Some(var_name.as_str().to_owned())]
                    .into_iter()
                    .chain(names)
                    .collect(),
                *inner,
            ),
            ast::Code::Match {
                idx,
                cases,
                els,
                span: _,
            } => {
                let mut next_case = self.visit_code(name, ns_idx, names.clone(), *els);

                for case in cases.into_iter().rev() {
                    let body = self.visit_code(name, ns_idx, names.clone(), case.body);
                    let cond = Sentence {
                        name: Some(name.to_owned()),
                        words: vec![
                            InnerWord::Copy(idx).into(),
                            case.value.into(),
                            InnerWord::Builtin(Builtin::Eq).into(),
                            Value::Pointer(vec![], body).into(),
                            Value::Pointer(vec![], next_case).into(),
                            Value::Symbol("if".to_owned()).into(),
                        ],
                    };

                    next_case = self.sentences.push_and_get_key(cond);
                }
                next_case
            }
        }
    }

    fn visit_sentence(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        names: VecDeque<Option<String>>,
        sentence: ast::Sentence<'t>,
    ) -> SentenceIndex {
        let s = self.convert_sentence(name, ns_idx, names, sentence);
        self.sentences.push_and_get_key(s)
    }

    fn convert_sentence(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        mut names: VecDeque<Option<String>>,
        sentence: ast::Sentence<'t>,
    ) -> Sentence<'t> {
        // let mut args: Option<VecDeque<Option<String>>> = sentence
        // .args
        // .map(|names| names.iter().rev().map(|s| Some(s.to_owned())).collect());

        // let mut names = match (args, names) {
        //         (None, None) => None,
        //         (Some(args), None) => Some(args),
        //         (None, Some(names)) => Some(names),
        //         (Some(args), Some(names)) => {
        //             Some(args.into_iter().chain(names).collect())
        //         }
        //     };
        let mut words = vec![];
        for e in sentence.exprs {
            for mut w in self.convert_expr(ns_idx, &mut names, e.into()) {
                w.names = Some(names.clone());
                match &w.inner {
                    InnerWord::Push(_) | InnerWord::Copy(_) => names.push_front(None),
                    InnerWord::Drop(idx) => {
                        names.remove(*idx);
                    }
                    InnerWord::Move(idx) => {
                        let moved = names.remove(*idx).unwrap();
                        names.push_front(moved);
                    }
                    InnerWord::Send(idx) => {
                        let moved = names.pop_front().unwrap();
                        names.insert(*idx, moved);
                    }
                    InnerWord::Builtin(builtin) => match builtin {
                        Builtin::Add
                        | Builtin::Eq
                        | Builtin::Curry
                        | Builtin::Or
                        | Builtin::And
                        | Builtin::Get
                        | Builtin::SymbolCharAt => {
                            names.pop_front();
                            names.pop_front();
                            names.push_front(None);
                        }
                        Builtin::NsEmpty => {
                            names.push_front(None);
                        }
                        Builtin::NsGet => {
                            let ns = names.pop_front().unwrap();
                            names.pop_front();
                            names.push_front(ns);
                            names.push_front(None);
                        }
                        Builtin::NsInsert => {
                            names.pop_front();
                            names.pop_front();
                            names.pop_front();
                            names.push_front(None);
                        }
                        Builtin::NsRemove => {
                            let ns = names.pop_front().unwrap();
                            names.pop_front();
                            names.push_front(ns);
                            names.push_front(None);
                        }
                        Builtin::Not | Builtin::SymbolLen | Builtin::IsCode => {
                            names.pop_front();
                            names.push_front(None);
                        }
                        Builtin::AssertEq => {
                            names.pop_front();
                            names.pop_front();
                        }
                    },
                }
                words.push(w)
            }
        }
        Sentence {
            name: Some(name.to_owned()),
            words,
        }
    }
    fn convert_expr(
        &mut self,
        ns_idx: NamespaceIndex,
        names: &VecDeque<Option<String>>,
        e: Expression<'t>,
    ) -> Vec<Word<'t>> {
        let mkword = |inner| Word {
            inner,
            span: Some(e.span),
            names: None,
        };
        match e.inner {
            InnerExpression::Path(segments) => segments
                .iter()
                .rev()
                .map(|s| Word {
                    inner: InnerWord::Push(Value::Symbol(s.as_str().to_owned())),
                    names: None,
                    span: Some(*s),
                })
                .chain([mkword(InnerWord::Push(Value::Namespace(ns_idx)))])
                .chain(
                    segments
                        .iter()
                        .map(|_| mkword(InnerWord::Builtin(Builtin::Get))),
                )
                .collect_vec(),
            InnerExpression::Literal(v) => vec![mkword(InnerWord::Push(v))],
            InnerExpression::FunctionLike(f, idx) => vec![mkword(match f.as_str() {
                "cp" => InnerWord::Copy(idx),
                "drop" => InnerWord::Drop(idx),
                "mv" => InnerWord::Move(idx),
                "sd" => InnerWord::Send(idx),
                _ => panic!("unknown reference: {}", f),
            })],
            InnerExpression::Reference(r) => {
                let Some(idx) = names.iter().position(|n| n.as_ref() == Some(&r)) else {
                    panic!("unknown reference: {}", r)
                };
                vec![mkword(InnerWord::Move(idx))]
            }
            InnerExpression::Delete(r) => {
                let Some(idx) = names.iter().position(|n| n.as_ref() == Some(&r)) else {
                    panic!("unknown reference: {}", r)
                };
                vec![mkword(InnerWord::Drop(idx))]
            }
            InnerExpression::Copy(r) => {
                let Some(idx) = names.iter().position(|n| n.as_ref() == Some(&r)) else {
                    panic!("unknown reference: {}", r)
                };
                vec![mkword(InnerWord::Copy(idx))]
            }
            InnerExpression::Builtin(name) => {
                if let Some(builtin) = Builtin::ALL.iter().find(|builtin| builtin.name() == name) {
                    vec![mkword(InnerWord::Builtin(*builtin))]
                } else {
                    panic!("unknown builtin: {}", name)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sentence<'t> {
    pub name: Option<String>,
    pub words: Vec<Word<'t>>,
}

macro_rules! builtins {
    {
        $(($ident:ident, $name:ident),)*
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Builtin {
            $($ident,)*
        }

        impl Builtin {
            pub const ALL: &[Builtin] = &[
                $(Builtin::$ident,)*
            ];

            pub fn name(self) -> &'static str {
                match self {
                    $(Builtin::$ident => stringify!($name),)*
                }
            }
        }
    };
}

builtins! {
    (Add, add),
    (Eq, eq),
    (AssertEq, assert_eq),
    (Curry, curry),
    (Or, or),
    (And, and),
    (Not, not),
    (IsCode, is_code),
    (Get, get),
    (SymbolCharAt, symbol_char_at),
    (SymbolLen, symbol_len),

    (NsEmpty, ns_empty),
    (NsInsert, ns_insert),
    (NsGet, ns_get),
    (NsRemove, ns_remove),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word<'t> {
    pub inner: InnerWord,
    pub span: Option<Span<'t>>,
    pub names: Option<VecDeque<Option<String>>>,
}

impl<'t> From<InnerWord> for Word<'t> {
    fn from(value: InnerWord) -> Self {
        Self {
            inner: value,
            span: None,
            names: None,
        }
    }
}
impl<'t> From<Value> for Word<'t> {
    fn from(value: Value) -> Self {
        Self {
            inner: InnerWord::Push(value),
            span: None,
            names: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InnerWord {
    Push(Value),
    Copy(usize),
    Drop(usize),
    Move(usize),
    Send(usize),
    Builtin(Builtin),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Symbol(String),
    Usize(usize),
    List(Vec<Value>),
    Pointer(Vec<Value>, SentenceIndex),
    Handle(usize),
    Bool(bool),
    Char(char),
    Namespace(NamespaceIndex),
    Namespace2(Namespace2),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Namespace2 {
    pub items: Vec<(String, Value)>,
}

pub struct ValueView<'a, 't> {
    pub lib: &'a Library<'t>,
    pub value: &'a Value,
}

impl<'a, 't> std::fmt::Display for ValueView<'a, 't> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Symbol(arg0) => write!(f, "@{}", arg0.replace("\n", "\\n")),
            Value::Usize(arg0) => write!(f, "{}", arg0),
            Value::List(arg0) => todo!(),
            Value::Handle(arg0) => todo!(),
            Value::Namespace(arg0) => write!(f, "ns({})", arg0.0),
            Value::Namespace2(arg0) => write!(f, "ns(TODO)"),
            Value::Bool(arg0) => write!(f, "{}", arg0),
            Value::Char(arg0) => write!(f, "'{}'", arg0),
            Value::Pointer(values, ptr) => {
                write!(
                    f,
                    "[{}]{}#{}",
                    values
                        .iter()
                        .map(|v| ValueView {
                            lib: self.lib,
                            value: v
                        })
                        .join(", "),
                    if *ptr == SentenceIndex::TRAP {
                        "TRAP"
                    } else {
                        if let Some(name) = &self.lib.sentences[*ptr].name {
                            name
                        } else {
                            "UNKNOWN"
                        }
                    },
                    ptr.0
                )
            }
        }
    }
}

impl Value {
    pub fn into_code<'a, 't>(self, lib: &'a Library<'t>) -> Option<(Vec<Value>, SentenceIndex)> {
        match self {
            Self::Pointer(values, ptr) => Some((values, ptr)),
            Value::Symbol(_)
            | Value::Usize(_)
            | Value::List(_)
            | Value::Namespace(_)
            | Value::Namespace2(_)
            | Value::Bool(_)
            | Value::Char(_)
            | Value::Handle(_) => None,
        }
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
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
    const NULL: Self = Type {
        arity_in: 0,
        arity_out: 0,
        judgements: vec![],
    };

    fn pad(&self) -> Self {
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

    pub fn compose(mut self, mut other: Self) -> Self {
        while self.arity_out < other.arity_in {
            self = self.pad()
        }
        while other.arity_in < self.arity_out {
            other = other.pad()
        }

        let mut res: Vec<Judgement> = vec![];
        for j1 in self.judgements {
            match j1 {
                Judgement::Eq(i1, o1) => {
                    for j2 in other.judgements.iter() {
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
                    for j2 in other.judgements.iter() {
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

        for j2 in other.judgements {
            match j2 {
                Judgement::Eq(i2, o2) => {}
                Judgement::OutExact(o, value) => res.push(Judgement::OutExact(o, value)),
            }
        }

        Type {
            arity_in: self.arity_in,
            arity_out: other.arity_out,
            judgements: res,
        }
    }
}

#[cfg(test)]
mod tests {}
