use derive_more::derive::{From, Into};
use typed_index_collections::TiVec;

use crate::ast::{self, Expression};

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct DeclIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct CodeIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SentenceIndex(usize);

#[derive(From, Into, Debug, Copy, Clone, PartialEq, Eq)]
pub struct WordIndex(usize);

#[derive(Debug, Clone, Default)]
pub struct Library {
    pub decls: TiVec<DeclIndex, Decl>,
    pub codes: TiVec<CodeIndex, Code>,
    pub sentences: TiVec<SentenceIndex, Sentence>,
    pub words: TiVec<WordIndex, Word>,

    pub code_to_decl: TiVec<CodeIndex, DeclIndex>,
}

impl Library {
    pub fn from_ast(lib: ast::Library) -> Self {
        let mut res = Self::default();
        res.fill(lib);
        res
    }

    fn fill(&mut self, lib: ast::Library) {
        self.decls = TiVec::new();
        for decl in lib.decls {
            self.visit_decl(decl);
        }

        self.code_to_decl = self.codes.iter().map(|_| DeclIndex(usize::MAX)).collect();
        for decl in self.decls.keys() {
            self.index_decl(decl);
        }
    }

    fn visit_decl(&mut self, decl: ast::Decl) -> DeclIndex {
        let new_code = self.visit_code(decl.value);
        self.decls.push_and_get_key(Decl {
            name: decl.name,
            code: new_code,
        })
    }

    fn visit_code(&mut self, code: ast::Code) -> CodeIndex {
        let new_code = match code {
            ast::Code::Sentence(sentence) => Code::Sentence(self.visit_sentence(sentence)),
            ast::Code::AndThen(sentence, code) => {
                Code::AndThen(self.visit_sentence(sentence), self.visit_code(*code))
            }
            ast::Code::If {
                cond,
                true_case,
                false_case,
            } => Code::If {
                cond: self.visit_sentence(cond),
                true_case: self.visit_code(*true_case),
                false_case: self.visit_code(*false_case),
            },
        };
        self.codes.push_and_get_key(new_code)
    }

    fn visit_sentence(&mut self, sentence: ast::Sentence) -> SentenceIndex {
        let new_sentence = Sentence(sentence.0.into_iter().map(|e| self.visit_expr(e)).collect());
        self.sentences.push_and_get_key(new_sentence)
    }

    fn visit_expr(&mut self, e: Expression) -> WordIndex {
        let w = match e {
            Expression::Symbol(v) => Word::Push(Value::Symbol(v)),
            Expression::Usize(v) => Word::Push(Value::Usize(v)),
            Expression::Bool(v) => Word::Push(Value::Bool(v)),
            Expression::FunctionLike("copy", idx) => Word::Copy(idx),
            Expression::FunctionLike("drop", idx) => Word::Drop(idx),
            Expression::FunctionLike("mv", idx) => Word::Move(idx),
            Expression::FunctionLike(name, _) => panic!("unknown reference: {}", name),
            Expression::Reference("add") => Word::Add,
            Expression::Reference("eq") => Word::Eq,
            Expression::Reference("curry") => Word::Curry,
            Expression::Reference(r) => {
                if let Some((decl_idx, decl)) = self
                    .decls
                    .iter_enumerated()
                    .find(|(_, decl)| decl.name == r)
                {
                    Word::PushDecl(decl_idx)
                } else {
                    panic!("unknown reference: {}", r)
                }
            }
        };
        self.words.push_and_get_key(w)
    }

    pub fn code_words(&self, code_idx: CodeIndex) -> Vec<(Word, Pointer)> {
        match &self.codes[code_idx] {
            Code::Sentence(sentence_idx) => self.sentence_words(*sentence_idx),
            Code::AndThen(sentence_idx, and_then) => std::iter::once((
                Word::Push(Value::Pointer(vec![], *and_then)),
                Pointer::Code(code_idx),
            ))
            .chain(self.sentence_words(*sentence_idx).into_iter())
            .collect(),
            Code::If {
                cond,
                true_case,
                false_case,
            } => self
                .sentence_words(*cond)
                .into_iter()
                .chain([
                    (
                        Word::Push(Value::Pointer(vec![], *true_case)),
                        Pointer::Code(code_idx),
                    ),
                    (
                        Word::Push(Value::Pointer(vec![], *false_case)),
                        Pointer::Code(code_idx),
                    ),
                    (Word::Push(Value::Symbol("if")), Pointer::Code(code_idx)),
                ])
                .collect(),
        }
    }

    fn sentence_words(&self, idx: SentenceIndex) -> Vec<(Word, Pointer)> {
        self.sentences[idx]
            .0
            .iter()
            .enumerate()
            .map(|(offset, widx)| (self.words[*widx].clone(), Pointer::Sentence(idx, offset)))
            .collect()
    }

    fn index_decl(&mut self, idx: DeclIndex) {
        self.index_code(idx, self.decls[idx].code)
    }

    fn index_code(&mut self, decl_idx: DeclIndex, code_idx: CodeIndex) {
        self.code_to_decl[code_idx] = decl_idx;
        match &self.codes[code_idx] {
            Code::Sentence(sentence_index) => {}
            Code::AndThen(sentence_index, code_idx) => self.index_code(decl_idx, *code_idx),
            &Code::If {
                cond,
                true_case,
                false_case,
            } => {
                self.index_code(decl_idx, true_case);
                self.index_code(decl_idx, false_case);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decl {
    pub name: String,
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

#[derive(Debug, Clone)]
pub enum Word {
    Add,
    Push(Value),
    PushDecl(DeclIndex),
    Cons,
    Snoc,
    Eq,
    Copy(usize),
    Drop(usize),
    Move(usize),
    Swap,
    Curry,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Symbol(&'static str),
    Usize(usize),
    List(Vec<Value>),
    // Quote(Box<Code>),
    Pointer(Vec<Value>, CodeIndex),
    Handle(usize),
    Bool(bool),
    Reference(String),
}

impl Value {
    pub fn into_code(self, lib: &Library) -> Option<(Vec<Value>, CodeIndex)> {
        match self {
            // Value::Quote(code) => Some(*code),
            Self::Pointer(values, ptr) => Some((values, ptr)),
            Value::Reference(name) => Some((
                vec![],
                lib.decls
                    .iter()
                    .enumerate()
                    .find_map(|(idx, d)| if d.name == name { Some(d.code) } else { None })
                    .unwrap(),
            )),
            Value::Symbol(_)
            | Value::Usize(_)
            | Value::List(_)
            | Value::Bool(_)
            | Value::Handle(_) => None,
        }
    }

    pub fn format(&self, mut f: impl std::fmt::Write, lib: &Library) -> std::fmt::Result {
        match self {
            Self::Reference(arg0) => write!(f, "{}", arg0),
            Self::Symbol(arg0) => write!(f, "*{}", arg0),
            Self::Usize(arg0) => write!(f, "{}", arg0),
            Self::List(arg0) => todo!(),
            Self::Handle(arg0) => todo!(),
            Self::Bool(arg0) => write!(f, "{}", arg0),
            Self::Pointer(values, ptr) => {
                let decl = &lib.decls[lib.code_to_decl[*ptr]];
                write!(f, "{:?}{}#{}", values, decl.name, ptr.0)
            }
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference(arg0) => write!(f, "{}", arg0),
            Self::Symbol(arg0) => write!(f, "*{}", arg0),
            Self::Usize(arg0) => write!(f, "{}", arg0),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            // Self::Quote(arg0) => write!(f, "{{{:?}}}", arg0),
            Self::Handle(arg0) => f.debug_tuple("Handle").field(arg0).finish(),
            Self::Bool(arg0) => write!(f, "{}", arg0),
            Self::Pointer(values, ptr) => write!(f, "[{:?}]({:?})", values, ptr),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pointer {
    Code(CodeIndex),
    Sentence(SentenceIndex, usize),
}
