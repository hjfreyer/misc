#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Library {
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub name: String,
    pub value: Code,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Symbol(&'static str),
    Reference(&'static str),
    FunctionLike(&'static str, usize),
    Usize(usize),
    Bool(bool),
}

impl From<bool> for Expression {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<usize> for Expression {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Code {
    Sentence(Sentence),
    AndThen(Sentence, Box<Code>),
    If {
        cond: Sentence,
        true_case: Box<Code>,
        false_case: Box<Code>,
    },
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence(pub Vec<Expression>);

impl Sentence {
    pub fn push(&mut self, s: impl Into<Sentence>) {
        for w in s.into().0 {
            self.0.push(w)
        }
    }
}
