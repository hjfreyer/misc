use std::{collections::HashMap, usize};

use derive_more::derive::{From, Into};
use itertools::Itertools;
use pest::Span;
use typed_index_collections::TiVec;

use crate::ast::{self, Expression, InnerExpression};

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CodeIndex(usize);

impl CodeIndex {
    pub const TRAP: Self = CodeIndex(usize::MAX);
}

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SentenceIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WordIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct NamespaceIndex(usize);

#[derive(Debug, Clone, Default)]
pub struct Library<'t> {
    pub namespaces: TiVec<NamespaceIndex, Namespace>,
    pub codes: TiVec<CodeIndex, Code>,
    pub sentences: TiVec<SentenceIndex, Sentence>,
    pub words: TiVec<WordIndex, Word<'t>>,

    pub code_to_name: TiVec<CodeIndex, String>,
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
    Code(CodeIndex),
    Namespace(NamespaceIndex),
}

impl<'t> Library<'t> {
    pub fn from_ast(lib: ast::Namespace<'t>) -> Self {
        let mut res = Self::default();
        res.visit_ns(lib);
        res
    }

    pub fn root_namespace(&self) -> NamespaceRef<'_, 't> {
        NamespaceRef {
            lib: self,
            idx: self.namespaces.first_key().unwrap(),
        }
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
                    let code_idx = self.visit_code(&decl.name, ns_idx, code);
                    self.namespaces[ns_idx]
                        .0
                        .push((decl.name, Entry::Code(code_idx)));
                }
            }
        }
        ns_idx
    }

    fn visit_code(&mut self, name: &str, ns_idx: NamespaceIndex, code: ast::Code<'t>) -> CodeIndex {
        let new_code = match code {
            ast::Code::Sentence(sentence) => {
                Code::Sentence(self.visit_sentence(name, ns_idx, sentence))
            }
            ast::Code::AndThen(sentence, code) => Code::AndThen(
                self.visit_sentence(name, ns_idx, sentence),
                self.visit_code(name, ns_idx, *code),
            ),
            ast::Code::If {
                cond,
                true_case,
                false_case,
            } => Code::If {
                cond: self.visit_sentence(name, ns_idx, cond),
                true_case: self.visit_code(name, ns_idx, *true_case),
                false_case: self.visit_code(name, ns_idx, *false_case),
            },
        };
        self.code_to_name.push(name.to_owned());
        self.codes.push_and_get_key(new_code)
    }

    fn visit_sentence(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        sentence: ast::Sentence<'t>,
    ) -> SentenceIndex {
        let new_sentence = Sentence(
            sentence
                .exprs
                .into_iter()
                .map(|e| self.visit_expr(ns_idx, e))
                .collect(),
        );
        self.sentences.push_and_get_key(new_sentence)
    }

    fn visit_expr(&mut self, ns_idx: NamespaceIndex, e: Expression<'t>) -> WordIndex {
        let w = match e.inner {
            InnerExpression::This => InnerWord::Push(Value::Namespace(ns_idx)),
            InnerExpression::Symbol(v) => InnerWord::Push(Value::Symbol(v)),
            InnerExpression::Usize(v) => InnerWord::Push(Value::Usize(v)),
            InnerExpression::Bool(v) => InnerWord::Push(Value::Bool(v)),
            InnerExpression::Char(v) => InnerWord::Push(Value::Char(v)),
            InnerExpression::FunctionLike(f, idx) => match f.as_str() {
                "cp" => InnerWord::Copy(idx),
                "drop" => InnerWord::Drop(idx),
                "mv" => InnerWord::Move(idx),
                _ => panic!("unknown reference: {}", f),
            },
            InnerExpression::Reference(r) => {
                panic!("unknown reference: {}", r)
            }
            InnerExpression::Builtin(name) => {
                if let Some(builtin) = Builtin::ALL.iter().find(|builtin| builtin.name() == name) {
                    InnerWord::Builtin(*builtin)
                } else {
                    panic!("unknown builtin: {}", name)
                }
            }
        };
        self.words.push_and_get_key(Word {
            inner: w,
            span: Some(e.span),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub name: &'static str,
    pub code: CodeIndex,
}

#[derive(Debug, Clone)]
pub enum Code {
    Sentence(SentenceIndex),
    AndThen(SentenceIndex, CodeIndex),
    If {
        cond: SentenceIndex,
        true_case: CodeIndex,
        false_case: CodeIndex,
    },
}

impl Code {}

#[derive(Debug, Clone)]
pub struct Sentence(pub Vec<WordIndex>);

macro_rules! builtins {
    {
        $(($ident:ident, $name:ident),)*
    } => {
        #[derive(Debug, Clone, Copy)]
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
}

// impl Builtin {
//     pub fn r#type(self) -> Type {
//         match self {
//             Builtin::Add => Type {
//                 arity_in: 2,
//                 arity_out: 1,
//                 judgements: vec![],
//             },
//             Builtin::Eq => todo!(),
//             Builtin::Curry => Type {
//                 arity_in: 2,
//                 arity_out: 1,
//                 judgements: vec![],
//             },
//             Builtin::Or => todo!(),
//             Builtin::And => todo!(),
//             Builtin::Not => todo!(),
//             Builtin::IsCode => todo!(),
//             Builtin::Get => todo!(),
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct Word<'t> {
    pub inner: InnerWord,
    pub span: Option<Span<'t>>,
}

#[derive(Debug, Clone)]
pub enum InnerWord {
    Push(Value),
    Copy(usize),
    Drop(usize),
    Move(usize),
    Builtin(Builtin),
}

impl InnerWord {
    // fn r#type(self) -> Type {
    //     match self {
    //         InnerWord::Push(value) => Type {
    //             arity_in: 0,
    //             arity_out: 1,
    //             judgements: vec![Judgement::OutExact(0, value.clone())],
    //         },
    //         InnerWord::Copy(idx) => Type {
    //             arity_in: idx + 1,
    //             arity_out: idx + 2,
    //             judgements: (0..(idx + 1))
    //                 .map(|i| Judgement::Eq(i, i + 1))
    //                 .chain(std::iter::once(Judgement::Eq(idx, 0)))
    //                 .collect(),
    //         },
    //         InnerWord::Drop(idx) => Type {
    //             arity_in: idx + 1,
    //             arity_out: idx,
    //             judgements: (0..idx).map(|i| Judgement::Eq(i, i)).collect(),
    //         },
    //         InnerWord::Move(idx) => Type {
    //             arity_in: idx + 1,
    //             arity_out: idx + 1,
    //             judgements: (0..idx)
    //                 .map(|i| Judgement::Eq(i, i + 1))
    //                 .chain(std::iter::once(Judgement::Eq(idx, 0)))
    //                 .collect(),
    //         },
    //         InnerWord::Builtin(builtin) => builtin.r#type(),
    //     }
    // }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Symbol(String),
    Usize(usize),
    List(Vec<Value>),
    Pointer(Vec<Value>, CodeIndex),
    Handle(usize),
    Bool(bool),
    Char(char),
    Namespace(NamespaceIndex),
}

pub struct ValueView<'a, 't> {
    pub lib: &'a Library<'t>,
    pub value: &'a Value,
}

impl<'a, 't> std::fmt::Display for ValueView<'a, 't> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value {
            Value::Symbol(arg0) => write!(f, "@{}", arg0),
            Value::Usize(arg0) => write!(f, "{}", arg0),
            Value::List(arg0) => todo!(),
            Value::Handle(arg0) => todo!(),
            Value::Namespace(arg0) => write!(f, "ns({})", arg0.0),
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
                    if *ptr == CodeIndex::TRAP {
                        "TRAP"
                    } else {
                        self.lib.code_to_name[*ptr].as_str()
                    },
                    ptr.0
                )
            }
        }
    }
}

impl Value {
    pub fn into_code<'a, 't>(self, lib: &'a Library<'t>) -> Option<(Vec<Value>, CodeIndex)> {
        match self {
            Self::Pointer(values, ptr) => Some((values, ptr)),
            Value::Symbol(_)
            | Value::Usize(_)
            | Value::List(_)
            | Value::Namespace(_)
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

// impl std::fmt::Debug for Value {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::Symbol(arg0) => write!(f, "@{}", arg0),
//             Self::Usize(arg0) => write!(f, "{}", arg0),
//             Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
//             Self::Handle(arg0) => f.debug_tuple("Handle").field(arg0).finish(),
//             Self::Bool(arg0) => write!(f, "{}", arg0),
//             Self::Pointer(values, ptr) => write!(f, "[{:?}]({:?})", values, ptr),
//             Self::Namespace(arg0) => f.debug_tuple("Namespace").field(arg0).finish(),
//         }
//     }
// }

#[derive(Clone, Copy)]
pub struct NamespaceRef<'a, 't> {
    pub lib: &'a Library<'t>,
    pub idx: NamespaceIndex,
}

impl<'a, 't> NamespaceRef<'a, 't> {
    pub fn entries(self) -> impl Iterator<Item = (&'a str, EntryView<'a, 't>)> + 'a {
        self.lib.namespaces[self.idx]
            .0
            .iter()
            .map(|(name, entry)| match entry {
                Entry::Code(code_index) => (
                    name.as_str(),
                    EntryView::Code(CodeRef {
                        lib: self.lib,
                        idx: *code_index,
                    }),
                ),
                Entry::Namespace(namespace_index) => (
                    name.as_str(),
                    EntryView::Namespace(NamespaceRef {
                        lib: self.lib,
                        idx: *namespace_index,
                    }),
                ),
            })
    }

    pub fn get(self, name: &str) -> Option<EntryView<'a, 't>> {
        self.entries()
            .find_map(|(n, e)| if name == n { Some(e) } else { None })
    }
}

#[derive(Clone, Copy)]
pub enum EntryView<'a, 't> {
    Code(CodeRef<'a, 't>),
    Namespace(NamespaceRef<'a, 't>),
}

#[derive(Clone, Copy)]
pub struct CodeRef<'a, 't> {
    pub lib: &'a Library<'t>,
    pub idx: CodeIndex,
}

#[derive(Clone, Copy)]
pub enum CodeView<'a, 't> {
    Sentence(SentenceRef<'a, 't>),
    AndThen(SentenceRef<'a, 't>, CodeRef<'a, 't>),
    If {
        cond: SentenceRef<'a, 't>,
        true_case: CodeRef<'a, 't>,
        false_case: CodeRef<'a, 't>,
    },
}

impl<'a, 't> CodeRef<'a, 't> {
    pub fn view(self) -> CodeView<'a, 't> {
        let code = &self.lib.codes[self.idx];
        match code {
            Code::Sentence(sentence_index) => CodeView::Sentence(SentenceRef {
                lib: self.lib,
                idx: *sentence_index,
            }),
            Code::AndThen(sentence_index, code_index) => CodeView::AndThen(
                SentenceRef {
                    lib: self.lib,
                    idx: *sentence_index,
                },
                CodeRef {
                    lib: self.lib,
                    idx: *code_index,
                },
            ),
            Code::If {
                cond,
                true_case,
                false_case,
            } => CodeView::If {
                cond: SentenceRef {
                    lib: self.lib,
                    idx: *cond,
                },
                true_case: CodeRef {
                    lib: self.lib,
                    idx: *true_case,
                },
                false_case: CodeRef {
                    lib: self.lib,
                    idx: *false_case,
                },
            },
        }
    }

    pub fn words(self) -> Vec<Word<'t>> {
        match self.view() {
            CodeView::Sentence(sentence) => sentence.words().collect(),
            CodeView::AndThen(sentence, and_then) => std::iter::once(
                (Word {
                    inner: InnerWord::Push(Value::Pointer(vec![], and_then.idx)),
                    span: None,
                }),
            )
            .chain(sentence.words().into_iter())
            .collect(),
            CodeView::If {
                cond,
                true_case,
                false_case,
            } => cond
                .words()
                .chain([
                    Word {
                        inner: InnerWord::Push(Value::Pointer(vec![], true_case.idx)),
                        span: None,
                    },
                    Word {
                        inner: InnerWord::Push(Value::Pointer(vec![], false_case.idx)),
                        span: None,
                    },
                    Word {
                        inner: InnerWord::Push(Value::Symbol("if".to_owned())),
                        span: None,
                    },
                ])
                .collect(),
        }
    }

    // pub fn r#type(self) -> Type {
    //     self.words()
    //         .into_iter()
    //         .map(|w| w.inner.r#type())
    //         .fold(Type::NULL, Type::compose)
    // }

    // pub fn eventual_type(self) -> Type {
    //     let mut t = self.r#type();

    //     if !t
    //         .judgements
    //         .iter()
    //         .any(|j| *j == Judgement::OutExact(0, Value::Symbol("exec".to_owned())))
    //     {
    //         return t;
    //     }

    //     let Some((next_push, next_code)) = t.judgements.iter().find_map(|j| match j {
    //         Judgement::OutExact(1, Value::Pointer(push, code)) => Some((push, *code)),
    //         _ => None,
    //     }) else {
    //         return t;
    //     };

    //     let next_type = pointer_type(self.lib, &next_push, next_code);

    //     t.compose(InnerWord::Drop(0).r#type())
    //         .compose(InnerWord::Drop(0).r#type())
    //         .compose(next_type)
    // }
}

// fn pointer_type(lib: &Library, push: &[Value], code: CodeIndex) -> Type {
//     let push_type = push
//         .iter()
//         .map(|v| InnerWord::Push(v.clone()).r#type())
//         .fold(Type::NULL, Type::compose);
//     push_type.compose(CodeRef { lib, idx: code }.r#type())
// }

#[derive(Clone, Copy)]
pub struct SentenceRef<'a, 't> {
    pub lib: &'a Library<'t>,
    pub idx: SentenceIndex,
}

impl<'a, 't> SentenceRef<'a, 't> {
    pub fn word_idxes(self) -> impl Iterator<Item = WordIndex> + 'a {
        self.lib.sentences[self.idx].0.iter().copied()
    }

    pub fn words(self) -> impl Iterator<Item = Word<'t>> + 'a {
        self.lib.sentences[self.idx]
            .0
            .iter()
            .enumerate()
            .map(move |(offset, widx)| (self.lib.words[*widx].clone()))
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
