use itertools::Itertools;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::flat::Value;

#[derive(Parser)]
#[grammar = "hanoi.pest"]
pub struct HanoiParser;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Module {
    pub namespace: Namespace,
}

impl Module {
    pub fn from_str(text: &str) -> anyhow::Result<Self> {
        let file = HanoiParser::parse(Rule::file, text)?;

        let file = file.exactly_one().unwrap();
        assert_eq!(file.as_rule(), Rule::file);

        let mut res = Namespace::default();
        for decl in file.into_inner() {
            match decl.as_rule() {
                Rule::decl => {
                    let (ident, code) = decl.into_inner().collect_tuple().unwrap();
                    assert_eq!(ident.as_rule(), Rule::identifier);
                    let code = Code::from_pair(code);

                    res.decls.push(Decl {
                        name: ident.as_str().to_owned(),
                        value: DeclValue::Code(code),
                    })
                }
                Rule::EOI => break,
                _ => unreachable!(),
            }
        }
        Ok(Module { namespace: res })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Namespace {
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl {
    pub name: String,
    pub value: DeclValue,
}

impl Decl {
    fn from_pair(mut p: Pair<Rule>) -> Decl {
        assert_eq!(p.as_rule(), Rule::decl);
        let (ident, code) = p.into_inner().collect_tuple().unwrap();
        assert_eq!(ident.as_rule(), Rule::identifier);
        let code = Code::from_pair(code);

        Decl {
            name: ident.as_str().to_owned(),
            value: DeclValue::Code(code),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclValue {
    Namespace(Namespace),
    Code(Code),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Symbol(String),
    Reference(String),
    FunctionLike(String, usize),
    Value(Value),
    Usize(usize),
    Bool(bool),
}

impl Expression {
    pub fn from_pair(p: pest::iterators::Pair<Rule>) -> Expression {
        assert_eq!(p.as_rule(), Rule::expr);

        let inner = p.into_inner().exactly_one().unwrap();
        match inner.as_rule() {
            Rule::literal => {
                let literal = inner.into_inner().exactly_one().unwrap();
                match literal.as_rule() {
                    Rule::int => Expression::Usize(literal.as_str().parse().unwrap()),
                    _ => unreachable!("{:?}", literal),
                }
            }
            Rule::identifier => Expression::Reference(inner.as_str().to_owned()),
            Rule::symbol => {
                let ident = inner.into_inner().exactly_one().unwrap();
                assert_eq!(ident.as_rule(), Rule::identifier);
                Expression::Symbol(ident.as_str().to_owned())
            }
            Rule::func_call => {
                let (fname, farg) = inner.into_inner().collect_tuple().unwrap();
                assert_eq!(fname.as_rule(), Rule::identifier);
                assert_eq!(farg.as_rule(), Rule::int);
                Expression::FunctionLike(fname.as_str().to_owned(), farg.as_str().parse().unwrap())
            }

            _ => unreachable!("{:?}", inner),
        }
    }
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

impl From<Value> for Expression {
    fn from(value: Value) -> Self {
        Self::Value(value)
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

impl Code {
    fn from_pair(p: pest::iterators::Pair<Rule>) -> Code {
        assert_eq!(p.as_rule(), Rule::code);
        let inner = p.into_inner().exactly_one().unwrap();
        match inner.as_rule() {
            Rule::sentence => Code::Sentence(Sentence::from_pair(inner)),
            Rule::and_then => {
                let (sentence, code) = inner.into_inner().collect_tuple().unwrap();
                assert_eq!(code.as_rule(), Rule::code);

                Code::AndThen(
                    Sentence::from_pair(sentence),
                    Box::new(Code::from_pair(code)),
                )
            }
            Rule::if_statement => {
                let (cond, true_case, false_case) = inner.into_inner().collect_tuple().unwrap();
                Code::If {
                    cond: Sentence::from_pair(cond),
                    true_case: Box::new(Code::from_pair(true_case)),
                    false_case: Box::new(Code::from_pair(false_case)),
                }
            }

            _ => unreachable!("{:?}", inner),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence(pub Vec<Expression>);

impl Sentence {
    pub fn push(&mut self, s: impl Into<Sentence>) {
        for w in s.into().0 {
            self.0.push(w)
        }
    }
}

impl Sentence {
    fn from_pair(p: pest::iterators::Pair<Rule>) -> Sentence {
        assert_eq!(p.as_rule(), Rule::sentence);

        Sentence(
            p.into_inner()
                .map(|word| {
                    assert_eq!(word.as_rule(), Rule::expr);
                    Expression::from_pair(word)
                })
                .collect(),
        )
    }
}
