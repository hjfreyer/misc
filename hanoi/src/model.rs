#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Symbol(&'static str),
    Usize(usize),
    List(Vec<Value>),
    Quote(Box<Code>),
    Handle(usize),
    Bool(bool),
    Reference(String),
}

impl Value {
    pub fn into_code(self, lib: &Library) -> Option<Code> {
        match self {
            Value::Quote(code) => Some(*code),
            Value::Reference(name) => Some(
                lib.decls
                    .iter()
                    .find_map(|d| {
                        if d.name == name {
                            let Value::Quote(code) = d.value.clone() else {
                                panic!()
                            };
                            Some(*code)
                        } else {
                            None
                        }
                    })
                    .unwrap(),
            ),
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
            Self::Quote(arg0) => write!(f, "{{{:?}}}", arg0),
            Self::Handle(arg0) => f.debug_tuple("Handle").field(arg0).finish(),
            Self::Bool(arg0) => write!(f, "{}", arg0),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Code {
    Sentence(Sentence),
    AndThen(Sentence, Box<Code>),
    Curried(Value, Box<Code>),
    If {
        cond: Sentence,
        true_case: Box<Code>,
        false_case: Box<Code>,
    },
}

impl Code {
    pub fn into_words(self) -> Vec<Word> {
        match self {
            Code::Sentence(sentence) => sentence.0,
            Code::AndThen(sentence, code) => {
                let mut res = vec![Word::Push(Value::Quote(code))];
                res.extend(sentence.0);
                res
            }
            Code::Curried(value, code) => {
                let mut res = vec![Word::Push(value)];
                res.extend(code.into_words());
                res
            }
            Code::If {
                cond,
                true_case,
                false_case,
            } => {
                let mut res = vec![];
                res.extend(cond.0);
                res.push(Word::Push(Value::Quote(true_case)));
                res.push(Word::Push(Value::Quote(false_case)));
                res.push(Word::Push(Value::Symbol("if")));
                res
            }
        }
    }
}

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sentence(arg0) => arg0.fmt(f),
            Self::AndThen(arg0, arg1) => write!(f, "{:?}; {:?}", arg0, arg1),
            Self::Curried(arg0, arg1) => write!(f, "[{:?}]({:?})", arg0, arg1),
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
        if let Some(Word::Push(Value::Quote(next))) = self.0.first() {
            let prefix = Sentence(self.0.iter().skip(1).cloned().collect());
            write!(f, "{:?}; {:?}", prefix, next)?;
        } else {
            for (i, w) in self.0.iter().enumerate() {
                if i != 0 {
                    write!(f, " ")?
                }
                match w {
                    Word::Add => write!(f, "add")?,
                    Word::Push(value) => write!(f, "{:?}", value)?,
                    Word::Cons => todo!(),
                    Word::Snoc => todo!(),
                    Word::Eq => write!(f, "eq")?,
                    Word::Copy(i) => write!(f, "copy({})", i)?,
                    Word::Drop(i) => write!(f, "drop({})", i)?,
                    Word::Move(i) => write!(f, "move({})", i)?,
                    Word::Swap => write!(f, "swap")?,
                    Word::Curry => write!(f, "curry")?,
                }
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
