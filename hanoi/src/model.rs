#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub decls: Vec<Decl>,
}

impl Library {
    pub fn words(&self, ptr: &LibPointer) -> (Vec<Word>, Option<LibPointer>) {
        let (words, decl_ptr) =
            self.decls[ptr.0].words(&|subptr| LibPointer(ptr.0, subptr), &ptr.1);
        (words, decl_ptr.map(|decl_ptr| LibPointer(ptr.0, decl_ptr)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub name: String,
    pub value: Code,
}
impl Decl {
    fn words(
        &self,
        mkptr: &impl Fn(CodePointer) -> LibPointer,
        ptr: &CodePointer,
    ) -> (Vec<Word>, Option<CodePointer>) {
        self.value.words(mkptr, ptr)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Symbol(&'static str),
    Usize(usize),
    List(Vec<Value>),
    // Quote(Box<Code>),
    Pointer(Vec<Value>, LibPointer),
    Handle(usize),
    Bool(bool),
    Reference(String),
}

impl Value {
    pub fn into_code(self, lib: &Library) -> Option<(Vec<Value>, LibPointer)> {
        match self {
            // Value::Quote(code) => Some(*code),
            Self::Pointer(values, ptr) => Some((values, ptr)),
            Value::Reference(name) => Some((
                vec![],
                lib.decls
                    .iter()
                    .enumerate()
                    .find_map(|(idx, d)| {
                        if d.name == name {
                            Some(LibPointer(idx, d.value.start_pointer()))
                        } else {
                            None
                        }
                    })
                    .unwrap(),
            )),
            Value::Symbol(_)
            | Value::Usize(_)
            | Value::List(_)
            | Value::Bool(_)
            | Value::Handle(_) => None,
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

#[derive(Clone, PartialEq, Eq)]
pub enum Code {
    Sentence(Sentence),
    AndThen(Sentence, Box<Code>),
    // Curried(Value, Box<Code>),
    If {
        cond: Sentence,
        true_case: Box<Code>,
        false_case: Box<Code>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibPointer(pub usize, pub CodePointer);

impl LibPointer {
    // pub fn next(self, lib: &Library) -> Option<LibPointer> {
    //     let code = self.1.next(&lib.decls[self.0].value)?;
    //     Self(self.0, code)
    // }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodePointer {
    Sentence(usize),
    AndThenContinue(Box<CodePointer>),
    // Curried(Box<CodePointer>),
    IfTrue(Box<CodePointer>),
    IfFalse(Box<CodePointer>),
}
impl CodePointer {
    // fn next(self, code: &Code) -> Option<CodePointer> {
    //     match (self, code) {
    //         (CodePointer::Sentence(idx), Code::Sentence(sentence)) => if idx+1 < sentence.0.len() {
    //             Some(CodePointer::Sentence(idx+1))
    //         } else {
    //             None
    //         },
    //         (CodePointer::Sentence(idx), Code::AndThen(sentence, code)) => todo!(),
    //         (CodePointer::Sentence(idx), Code::If { cond, true_case, false_case }) => todo!(),
    //         (CodePointer::AndThenContinue(code_pointer), Code::Sentence(sentence)) => todo!(),
    //         (CodePointer::AndThenContinue(code_pointer), Code::AndThen(sentence, code)) => todo!(),
    //         (CodePointer::AndThenContinue(code_pointer), Code::If { cond, true_case, false_case }) => todo!(),
    //         (CodePointer::Curried(code_pointer), Code::Sentence(sentence)) => todo!(),
    //         (CodePointer::Curried(code_pointer), Code::AndThen(sentence, code)) => todo!(),
    //         (CodePointer::Curried(code_pointer), Code::If { cond, true_case, false_case }) => todo!(),
    //         (CodePointer::IfTrue(code_pointer), Code::Sentence(sentence)) => todo!(),
    //         (CodePointer::IfTrue(code_pointer), Code::AndThen(sentence, code)) => todo!(),
    //         (CodePointer::IfTrue(code_pointer), Code::If { cond, true_case, false_case }) => todo!(),
    //         (CodePointer::IfFalse(code_pointer), Code::Sentence(sentence)) => todo!(),
    //         (CodePointer::IfFalse(code_pointer), Code::AndThen(sentence, code)) => todo!(),
    //         (CodePointer::IfFalse(code_pointer), Code::If { cond, true_case, false_case }) => todo!(),
    //     }
    // }
}

impl Code {
    // pub fn into_words(self) -> Vec<Word> {
    //     match self {
    //         Code::Sentence(sentence) => sentence.0,
    //         Code::AndThen(sentence, code) => {
    //             let mut res = vec![Word::Push(Value::Quote(code))];
    //             res.extend(sentence.0);
    //             res
    //         }
    //         Code::Curried(value, code) => {
    //             let mut res = vec![Word::Push(value)];
    //             res.extend(code.into_words());
    //             res
    //         }
    //         Code::If {
    //             cond,
    //             true_case,
    //             false_case,
    //         } => {
    //             let mut res = vec![];
    //             res.extend(cond.0);
    //             res.push(Word::Push(Value::Quote(true_case)));
    //             res.push(Word::Push(Value::Quote(false_case)));
    //             res.push(Word::Push(Value::Symbol("if")));
    //             res
    //         }
    //     }
    // }

    fn words(
        &self,
        mkptr: &dyn Fn(CodePointer) -> LibPointer,
        ptr: &CodePointer,
    ) -> (Vec<Word>, Option<CodePointer>) {
        dbg!(ptr);
        match (self, ptr) {
            (Code::Sentence(sentence), CodePointer::Sentence(idx)) => {
                let words = vec![sentence.0[*idx].clone()];
                if idx + 1 == sentence.0.len() {
                    (words, None)
                } else {
                    (words, Some(CodePointer::Sentence(idx + 1)))
                }
            }
            (Code::Sentence(sentence), _) => panic!(),
            (Code::AndThen(sentence, code), CodePointer::Sentence(idx)) => {
                let mut words = vec![];
                if *idx == 0 {
                    words.push(Word::Push(Value::Pointer(
                        vec![],
                        mkptr(CodePointer::AndThenContinue(Box::new(code.start_pointer()))),
                    )))
                }
                words.push(sentence.0[*idx].clone());
                if idx + 1 == sentence.0.len() {
                    (words, None)
                } else {
                    (words, Some(CodePointer::Sentence(idx + 1)))
                }
            }
            (Code::AndThen(sentence, code), CodePointer::AndThenContinue(code_pointer)) => {
                let (words, ptr) = code.words(
                    &|subptr| mkptr(CodePointer::AndThenContinue(Box::new(subptr))),
                    code_pointer,
                );
                (
                    words,
                    ptr.map(|p| CodePointer::AndThenContinue(Box::new(p))),
                )
            }
            (Code::AndThen(sentence, code), _) => panic!(),
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                CodePointer::Sentence(idx),
            ) => {
                let mut words = vec![];
                words.push(cond.0[*idx].clone());
                if idx + 1 == cond.0.len() {
                    words.push(Word::Push(Value::Pointer(
                        vec![],
                        mkptr(CodePointer::IfTrue(Box::new(true_case.start_pointer()))),
                    )));
                    words.push(Word::Push(Value::Pointer(
                        vec![],
                        mkptr(CodePointer::IfFalse(Box::new(false_case.start_pointer()))),
                    )));
                    words.push(Word::Push(Value::Symbol("if")));
                    (words, None)
                } else {
                    (words, Some(CodePointer::Sentence(idx + 1)))
                }
            }
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                CodePointer::IfTrue(code_pointer),
            ) => {
                let (words, ptr) = true_case.words(
                    &|subptr| mkptr(CodePointer::IfTrue(Box::new(subptr))),
                    code_pointer,
                );
                (
                    words,
                    ptr.map(|subptr| CodePointer::IfTrue(Box::new(subptr))),
                )
            }
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                CodePointer::IfFalse(code_pointer),
            ) => {
                let (words, ptr) = false_case.words(
                    &|subptr| mkptr(CodePointer::IfFalse(Box::new(subptr))),
                    code_pointer,
                );
                (
                    words,
                    ptr.map(|subptr| CodePointer::IfFalse(Box::new(subptr))),
                )
            }
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                _,
            ) => panic!(),
        }
    }

    pub fn start_pointer(&self) -> CodePointer {
        match self {
            Code::Sentence(sentence) => CodePointer::Sentence(0),
            Code::AndThen(sentence, code) => CodePointer::Sentence(0),
            // Code::Curried(value, code) => CodePointer::AndThenContinue(Box::new(code.start_pointer())),
            Code::If {
                cond,
                true_case,
                false_case,
            } => CodePointer::Sentence(0),
        }
    }
}

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentence(arg0) => arg0.fmt(f),
            Self::AndThen(arg0, arg1) => write!(f, "{:?}; {:?}", arg0, arg1),
            // Self::Curried(arg0, arg1) => write!(f, "[{:?}]({:?})", arg0, arg1),
            Self::If {
                cond,
                true_case,
                false_case,
            } => write!(
                f,
                "{:?} if {{ {:?} }} else {{ {:?} }}",
                cond, true_case, false_case
            ),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Sentence(pub Vec<Word>);

impl Sentence {
    pub fn push(&mut self, s: impl Into<Sentence>) {
        for w in s.into().0 {
            self.0.push(w)
        }
    }
}

impl std::fmt::Debug for Sentence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(Word::Push(Value::Pointer(values, next))) = self.0.first() {
            let prefix = Sentence(self.0.iter().skip(1).cloned().collect());
            write!(f, "{:?}; {:?}", prefix, next)?;
        } else {
            for (i, w) in self.0.iter().enumerate() {
                if i != 0 {
                    write!(f, " ")?
                }
                write!(f, "{:?}", w)?;
            }
        }
        Ok(())
    }
}

impl From<Word> for Sentence {
    fn from(value: Word) -> Self {
        {
            let w = value;
            Sentence(vec![w])
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub enum Word {
    Add,
    Push(Value),
    Cons,
    Snoc,
    Eq,
    Copy(usize),
    Drop(usize),
    Move(usize),
    Swap,
    Curry,
}

impl std::fmt::Debug for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Word::Add => write!(f, "add"),
            Word::Push(value) => write!(f, "{:?}", value),
            Word::Cons => todo!(),
            Word::Snoc => todo!(),
            Word::Eq => write!(f, "eq"),
            Word::Copy(i) => write!(f, "copy({})", i),
            Word::Drop(i) => write!(f, "drop({})", i),
            Word::Move(i) => write!(f, "mv({})", i),
            Word::Swap => write!(f, "swap"),
            Word::Curry => write!(f, "curry"),
        }
    }
}

pub struct LibAndPointer {
    pub decls: Vec<DeclAndPointer>,
}

impl LibAndPointer {
    pub fn new(lib: Library, ptr: Option<LibPointer>) -> Self {
        Self {
            decls: lib
                .decls
                .into_iter()
                .enumerate()
                .map(|(idx, decl)| {
                    DeclAndPointer(
                        decl.name,
                        CodeAndPointer::new(
                            decl.value,
                            match &ptr {
                                Some(ptr) if ptr.0 == idx => Some(ptr.1.clone()),
                                _ => None,
                            },
                        ),
                    )
                })
                .collect(),
        }
    }
}

pub struct DeclAndPointer(pub String, pub CodeAndPointer);

impl CodeAndPointer {
    pub fn new(code: Code, ptr: Option<CodePointer>) -> Self {
        match (code, ptr) {
            (Code::Sentence(sentence), None) => {
                CodeAndPointer::Sentence(SentenceAndPointer(sentence, None))
            }
            (Code::Sentence(sentence), Some(CodePointer::Sentence(idx))) => {
                CodeAndPointer::Sentence(SentenceAndPointer(sentence, Some(idx)))
            }
            (Code::Sentence(sentence), _) => panic!(),
            (Code::AndThen(sentence, code), None) => CodeAndPointer::AndThen(
                SentenceAndPointer(sentence, None),
                Box::new(CodeAndPointer::new(*code, None)),
            ),
            (Code::AndThen(sentence, code), Some(CodePointer::Sentence(idx))) => {
                CodeAndPointer::AndThen(
                    SentenceAndPointer(sentence, Some(idx)),
                    Box::new(CodeAndPointer::new(*code, None)),
                )
            }
            (Code::AndThen(sentence, code), Some(CodePointer::AndThenContinue(code_pointer))) => {
                CodeAndPointer::AndThen(
                    SentenceAndPointer(sentence, None),
                    Box::new(CodeAndPointer::new(*code, Some(*code_pointer))),
                )
            }
            (Code::AndThen(sentence, code), _) => panic!(),
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                None,
            ) => CodeAndPointer::If {
                cond: SentenceAndPointer(cond, None),
                true_case: Box::new(CodeAndPointer::new(*true_case, None)),
                false_case: Box::new(CodeAndPointer::new(*false_case, None)),
            },
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                Some(CodePointer::Sentence(idx)),
            ) => CodeAndPointer::If {
                cond: SentenceAndPointer(cond, Some(idx)),
                true_case: Box::new(CodeAndPointer::new(*true_case, None)),
                false_case: Box::new(CodeAndPointer::new(*false_case, None)),
            },
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                Some(CodePointer::IfTrue(code_pointer)),
            ) => CodeAndPointer::If {
                cond: SentenceAndPointer(cond, None),
                true_case: Box::new(CodeAndPointer::new(*true_case, Some(*code_pointer))),
                false_case: Box::new(CodeAndPointer::new(*false_case, None)),
            },
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                Some(CodePointer::IfFalse(code_pointer)),
            ) => CodeAndPointer::If {
                cond: SentenceAndPointer(cond, None),
                true_case: Box::new(CodeAndPointer::new(*true_case, None)),
                false_case: Box::new(CodeAndPointer::new(*false_case, Some(*code_pointer))),
            },
            (
                Code::If {
                    cond,
                    true_case,
                    false_case,
                },
                _,
            ) => panic!(),
        }
    }
}

pub enum CodeAndPointer {
    Sentence(SentenceAndPointer),
    AndThen(SentenceAndPointer, Box<CodeAndPointer>),
    If {
        cond: SentenceAndPointer,
        true_case: Box<CodeAndPointer>,
        false_case: Box<CodeAndPointer>,
    },
}

pub struct SentenceAndPointer(pub Sentence, pub Option<usize>);

pub struct PrettyPrinter<W> {
    pub f: W,
    pub indent: String,
}

impl<W: std::fmt::Write> PrettyPrinter<W> {
    pub fn print_lib(&mut self, lib: &Library) -> std::fmt::Result {
        for decl in lib.decls.iter() {
            self.print_decl(decl)?;
        }
        Ok(())
    }

    fn print_decl(&mut self, decl: &Decl) -> std::fmt::Result {
        writeln!(self.f, "{}let {} = {{", self.indent, decl.name)?;
        self.indent += "  ";
        self.print_code(&decl.value)?;
        self.indent.truncate(self.indent.len() - 2);
        writeln!(self.f, "{}}};\n", self.indent)?;
        Ok(())
    }

    fn print_code(&mut self, value: &Code) -> std::fmt::Result {
        match value {
            Code::Sentence(sentence) => writeln!(self.f, "{}{:?}", self.indent, sentence)?,
            Code::AndThen(sentence, code) => {
                writeln!(self.f, "{}{:?};", self.indent, sentence)?;
                self.print_code(code)?;
            }
            Code::If {
                cond,
                true_case,
                false_case,
            } => {
                writeln!(self.f, "{}{:?} if {{", self.indent, cond)?;
                self.indent += "  ";

                self.indent.truncate(self.indent.len() - 2);
                writeln!(self.f, "{}}} else {{", self.indent)?;
                self.indent += "  ";

                self.indent.truncate(self.indent.len() - 2);
                writeln!(self.f, "{}}};", self.indent)?;
            }
        }
        Ok(())
    }
}
