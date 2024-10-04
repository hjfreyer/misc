use std::collections::HashMap;

use derive_more::derive::{From, Into};
use typed_index_collections::TiVec;

use crate::ast::{self, Expression};

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CodeIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SentenceIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WordIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct NamespaceIndex(usize);

#[derive(Debug, Clone, Default)]
pub struct Library {
    pub namespaces: TiVec<NamespaceIndex, Namespace>,
    pub codes: TiVec<CodeIndex, Code>,
    pub sentences: TiVec<SentenceIndex, Sentence>,
    pub words: TiVec<WordIndex, Word>,
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

impl Library {
    pub fn from_ast(lib: ast::Namespace) -> Self {
        let mut res = Self::default();
        res.visit_ns(lib);
        res
    }

    pub fn root_namespace(&self) -> NamespaceRef<'_> {
        NamespaceRef {
            lib: self,
            idx: self.namespaces.first_key().unwrap(),
        }
    }

    fn visit_ns(&mut self, ns: ast::Namespace) -> NamespaceIndex {
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
                    let code_idx = self.visit_code(ns_idx, code);
                    self.namespaces[ns_idx]
                        .0
                        .push((decl.name, Entry::Code(code_idx)));
                }
            }
        }
        ns_idx
    }

    fn visit_code(&mut self, ns_idx: NamespaceIndex, code: ast::Code) -> CodeIndex {
        let new_code = match code {
            ast::Code::Sentence(sentence) => Code::Sentence(self.visit_sentence(ns_idx, sentence)),
            ast::Code::AndThen(sentence, code) => Code::AndThen(
                self.visit_sentence(ns_idx, sentence),
                self.visit_code(ns_idx, *code),
            ),
            ast::Code::If {
                cond,
                true_case,
                false_case,
            } => Code::If {
                cond: self.visit_sentence(ns_idx, cond),
                true_case: self.visit_code(ns_idx, *true_case),
                false_case: self.visit_code(ns_idx, *false_case),
            },
        };
        self.codes.push_and_get_key(new_code)
    }

    fn visit_sentence(&mut self, ns_idx: NamespaceIndex, sentence: ast::Sentence) -> SentenceIndex {
        let new_sentence = Sentence(
            sentence
                .0
                .into_iter()
                .map(|e| self.visit_expr(ns_idx, e))
                .collect(),
        );
        self.sentences.push_and_get_key(new_sentence)
    }

    fn visit_expr(&mut self, ns_idx: NamespaceIndex, e: Expression) -> WordIndex {
        let w = match e {
            Expression::This => Word::Push(Value::Namespace(ns_idx)),
            Expression::Symbol(v) => Word::Push(Value::Symbol(v)),
            Expression::Usize(v) => Word::Push(Value::Usize(v)),
            Expression::Bool(v) => Word::Push(Value::Bool(v)),
            Expression::Value(v) => Word::Push(v),
            Expression::FunctionLike(f, idx) => match f.as_str() {
                "copy" => Word::Copy(idx),
                "drop" => Word::Drop(idx),
                "mv" => Word::Move(idx),
                _ => panic!("unknown reference: {}", f),
            },
            Expression::Reference(r) => {
                if let Some(builtin) = Builtin::ALL.iter().find(|builtin| builtin.name() == r) {
                    Word::Builtin(*builtin)
                } else {
                    // if let Some((decl_idx, decl)) = self
                    //     .decls
                    //     .iter_enumerated()
                    //     .find(|(_, decl)| decl.name == r)
                    // {
                    //     Word::Push(Value::Pointer(vec![], decl.code))
                    // } else {
                    panic!("unknown reference: {}", r)
                    // }
                }
            }
        };
        self.words.push_and_get_key(w)
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
    (Curry, curry),
    (Or, or),
    (And, and),
    (Not, not),
    (IsCode, is_code),
    (Get, get),
}

impl Builtin {
    pub fn r#type(self) -> Type {
        match self {
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
            Builtin::Get => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Word {
    Push(Value),
    Copy(usize),
    Drop(usize),
    Move(usize),
    Builtin(Builtin),
}

impl Word {
    fn r#type(self) -> Type {
        match self {
            Word::Push(value) => Type {
                arity_in: 0,
                arity_out: 1,
                judgements: vec![Judgement::OutExact(0, value.clone())],
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
            Word::Builtin(builtin) => builtin.r#type(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Symbol(String),
    Usize(usize),
    List(Vec<Value>),
    Pointer(Vec<Value>, CodeIndex),
    Handle(usize),
    Bool(bool),
    Namespace(NamespaceIndex),
}

impl Value {
    pub fn into_code(self, lib: &Library) -> Option<(Vec<Value>, CodeRef)> {
        match self {
            Self::Pointer(values, ptr) => Some((values, CodeRef { lib, idx: ptr })),
            Value::Symbol(_)
            | Value::Usize(_)
            | Value::List(_)
            | Value::Namespace(_)
            | Value::Bool(_)
            | Value::Handle(_) => None,
        }
    }

    pub fn format(&self, mut f: impl std::fmt::Write, lib: &Library) -> std::fmt::Result {
        match self {
            Self::Symbol(arg0) => write!(f, "*{}", arg0),
            Self::Usize(arg0) => write!(f, "{}", arg0),
            Self::List(arg0) => todo!(),
            Self::Handle(arg0) => todo!(),
            Self::Namespace(arg0) => write!(f, "ns({})", arg0.0),
            Self::Bool(arg0) => write!(f, "{}", arg0),
            Self::Pointer(values, ptr) => {
                // let decl = &lib.decls[lib.code_to_decl[*ptr]];
                // write!(f, "{:?}{}#{}", values, decl.name, ptr.0)
                write!(f, "ptr({})", ptr.0)
            }
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Symbol(arg0) => write!(f, "*{}", arg0),
            Self::Usize(arg0) => write!(f, "{}", arg0),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Handle(arg0) => f.debug_tuple("Handle").field(arg0).finish(),
            Self::Bool(arg0) => write!(f, "{}", arg0),
            Self::Pointer(values, ptr) => write!(f, "[{:?}]({:?})", values, ptr),
            Self::Namespace(arg0) => f.debug_tuple("Namespace").field(arg0).finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pointer {
    Code(CodeIndex),
    Sentence(SentenceIndex, usize),
}

#[derive(Clone, Copy)]
pub struct NamespaceRef<'a> {
    pub lib: &'a Library,
    pub idx: NamespaceIndex,
}

impl<'a> NamespaceRef<'a> {
    pub fn entries(self) -> impl Iterator<Item = (&'a str, EntryView<'a>)> + 'a {
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

    pub fn get(self, name: &str) -> Option<EntryView<'a>> {
        self.entries()
            .find_map(|(n, e)| if name == n { Some(e) } else { None })
    }
}

#[derive(Clone, Copy)]
pub enum EntryView<'a> {
    Code(CodeRef<'a>),
    Namespace(NamespaceRef<'a>),
}

#[derive(Clone, Copy)]
pub struct CodeRef<'a> {
    pub lib: &'a Library,
    pub idx: CodeIndex,
}

#[derive(Clone, Copy)]
pub enum CodeView<'a> {
    Sentence(SentenceRef<'a>),
    AndThen(SentenceRef<'a>, CodeRef<'a>),
    If {
        cond: SentenceRef<'a>,
        true_case: CodeRef<'a>,
        false_case: CodeRef<'a>,
    },
}

impl<'a> CodeRef<'a> {
    pub fn view(self) -> CodeView<'a> {
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

    pub fn words(self) -> Vec<(Word, Pointer)> {
        match self.view() {
            CodeView::Sentence(sentence) => sentence.words().collect(),
            CodeView::AndThen(sentence, and_then) => std::iter::once((
                Word::Push(Value::Pointer(vec![], and_then.idx)),
                Pointer::Code(self.idx),
            ))
            .chain(sentence.words().into_iter())
            .collect(),
            CodeView::If {
                cond,
                true_case,
                false_case,
            } => cond
                .words()
                .chain([
                    (
                        Word::Push(Value::Pointer(vec![], true_case.idx)),
                        Pointer::Code(self.idx),
                    ),
                    (
                        Word::Push(Value::Pointer(vec![], false_case.idx)),
                        Pointer::Code(self.idx),
                    ),
                    (
                        Word::Push(Value::Symbol("if".to_owned())),
                        Pointer::Code(self.idx),
                    ),
                ])
                .collect(),
        }
    }

    pub fn r#type(self) -> Type {
        self.words()
            .into_iter()
            .map(|(w, p)| w.r#type())
            .fold(Type::NULL, Type::compose)
    }

    pub fn eventual_type(self) -> Type {
        let mut t = self.r#type();

        if !t
            .judgements
            .iter()
            .any(|j| *j == Judgement::OutExact(0, Value::Symbol("exec".to_owned())))
        {
            return t;
        }

        let Some((next_push, next_code)) = t.judgements.iter().find_map(|j| match j {
            Judgement::OutExact(1, Value::Pointer(push, code)) => Some((push, *code)),
            _ => None,
        }) else {
            return t;
        };

        let next_type = pointer_type(self.lib, &next_push, next_code);

        t.compose(Word::Drop(0).r#type())
            .compose(Word::Drop(0).r#type())
            .compose(next_type)
    }
}

fn pointer_type(lib: &Library, push: &[Value], code: CodeIndex) -> Type {
    let push_type = push
        .iter()
        .map(|v| Word::Push(v.clone()).r#type())
        .fold(Type::NULL, Type::compose);
    push_type.compose(CodeRef { lib, idx: code }.r#type())
}

#[derive(Clone, Copy)]
pub struct SentenceRef<'a> {
    pub lib: &'a Library,
    pub idx: SentenceIndex,
}

impl<'a> SentenceRef<'a> {
    pub fn word_idxes(self) -> impl Iterator<Item = WordIndex> + 'a {
        self.lib.sentences[self.idx].0.iter().copied()
    }

    pub fn words(self) -> impl Iterator<Item = (Word, Pointer)> + 'a {
        self.lib.sentences[self.idx]
            .0
            .iter()
            .enumerate()
            .map(move |(offset, widx)| {
                (
                    self.lib.words[*widx].clone(),
                    Pointer::Sentence(self.idx, offset),
                )
            })
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
