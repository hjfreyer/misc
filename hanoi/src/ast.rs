use itertools::Itertools;
use pest::{iterators::Pair, Parser, Span};
use pest_derive::Parser;

use crate::flat::Value;

#[derive(Parser)]
#[grammar = "hanoi.pest"]
pub struct HanoiParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module<'t> {
    pub namespace: Namespace<'t>,
}

impl<'t> Module<'t> {
    pub fn from_str(text: &'t str) -> anyhow::Result<Self> {
        let file = HanoiParser::parse(Rule::file, text)?;

        let file = file.exactly_one().unwrap();
        assert_eq!(file.as_rule(), Rule::file);

        let (ns, eoi) = file.into_inner().collect_tuple().unwrap();
        assert_eq!(eoi.as_rule(), Rule::EOI);

        Ok(Module {
            namespace: Namespace::from_pair(ns),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace<'t> {
    pub decls: Vec<Decl<'t>>,
    pub span: Span<'t>,
}

fn ident_from_pair<'t>(p: pest::iterators::Pair<'t, Rule>) -> &'t str {
    assert_eq!(p.as_rule(), Rule::identifier);
    p.as_str()
}

impl<'t> Namespace<'t> {
    fn from_pair(p: pest::iterators::Pair<'t, Rule>) -> Self {
        assert_eq!(p.as_rule(), Rule::namespace);

        let mut res = Self {
            decls: vec![],
            span: p.as_span(),
        };
        for decl in p.into_inner() {
            match decl.as_rule() {
                Rule::code_decl => {
                    let (ident, code) = decl.into_inner().collect_tuple().unwrap();
                    assert_eq!(ident.as_rule(), Rule::identifier);
                    let code = Code::from_pair(code);

                    res.decls.push(Decl {
                        name: ident.as_str().to_owned(),
                        value: DeclValue::Code(code),
                    })
                }
                Rule::ns_decl => {
                    let (ident, ns) = decl.into_inner().collect_tuple().unwrap();
                    assert_eq!(ident.as_rule(), Rule::identifier);
                    let ns = Namespace::from_pair(ns);

                    res.decls.push(Decl {
                        name: ident.as_str().to_owned(),
                        value: DeclValue::Namespace(ns),
                    })
                }
                _ => unreachable!(),
            }
        }
        res
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decl<'t> {
    pub name: String,
    pub value: DeclValue<'t>,
}

impl<'t> Decl<'t> {
    fn from_pair(mut p: Pair<'t, Rule>) -> Decl<'t> {
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
pub enum DeclValue<'t> {
    Namespace(Namespace<'t>),
    Code(Code<'t>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression<'t> {
    pub span: Span<'t>,
    pub inner: InnerExpression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InnerExpression {
    Symbol(String),
    Builtin(String),
    Reference(String),
    Delete(String),
    Copy(String),
    FunctionLike(String, usize),
    Usize(usize),
    Bool(bool),
    Char(char),
    This,
}

impl<'t> Expression<'t> {
    pub fn from_pair(p: pest::iterators::Pair<'t, Rule>) -> Self {
        assert_eq!(p.as_rule(), Rule::expr);

        let span = p.as_span();
        let child = p.into_inner().exactly_one().unwrap();
        let inner = match child.as_rule() {
            Rule::literal => {
                let literal = child.into_inner().exactly_one().unwrap();
                match literal.as_rule() {
                    Rule::int => InnerExpression::Usize(literal.as_str().parse().unwrap()),
                    Rule::bool => InnerExpression::Bool(literal.as_str().parse().unwrap()),
                    Rule::char_lit => {
                        let chr = literal.into_inner().exactly_one().unwrap();
                        assert_eq!(Rule::lit_char, chr.as_rule());

                        let c = match chr.as_str() {
                            "\\n" => '\n',
                            c => c.chars().exactly_one().unwrap(),
                        };

                        InnerExpression::Char(c)
                    }
                    _ => unreachable!("{:?}", literal),
                }
            }
            Rule::identifier => InnerExpression::Reference(child.as_str().to_owned()),
            Rule::delete => InnerExpression::Delete(
                ident_from_pair(child.into_inner().exactly_one().unwrap()).to_owned(),
            ),
            Rule::copy => InnerExpression::Copy(
                ident_from_pair(child.into_inner().exactly_one().unwrap()).to_owned(),
            ),
            Rule::symbol => {
                let ident = child.into_inner().exactly_one().unwrap();
                match ident.as_rule() {
                    Rule::identifier => InnerExpression::Symbol(ident.as_str().to_owned()),
                    Rule::string => {
                        let inner = ident.into_inner().exactly_one().unwrap();
                        assert_eq!(inner.as_rule(), Rule::str_inner);

                        InnerExpression::Symbol(
                            inner.as_str().replace("\\n", "\n").replace("\\\"", "\""),
                        )
                    }
                    _ => unreachable!(),
                }
            }
            Rule::builtin => {
                let ident = child.into_inner().exactly_one().unwrap();
                assert_eq!(ident.as_rule(), Rule::identifier);
                InnerExpression::Builtin(ident.as_str().to_owned())
            }
            Rule::func_call => {
                let (fname, farg) = child.into_inner().collect_tuple().unwrap();
                assert_eq!(fname.as_rule(), Rule::identifier);
                assert_eq!(farg.as_rule(), Rule::int);
                InnerExpression::FunctionLike(
                    fname.as_str().to_owned(),
                    farg.as_str().parse().unwrap(),
                )
            }
            Rule::this => InnerExpression::This,
            _ => unreachable!("{:?}", child),
        };
        Self { span, inner }
    }
}

impl From<bool> for InnerExpression {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<usize> for InnerExpression {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Code<'t> {
    Sentence(Sentence<'t>),
    AndThen(Sentence<'t>, Box<Code<'t>>),
    If {
        cond: Sentence<'t>,
        true_case: Box<Code<'t>>,
        false_case: Box<Code<'t>>,
    },
}

impl<'t> Code<'t> {
    fn from_pair(p: pest::iterators::Pair<'t, Rule>) -> Self {
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

impl<'t> std::fmt::Debug for Code<'t> {
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
pub struct Sentence<'t> {
    pub args: Option<Vec<String>>,
    pub exprs: Vec<Expression<'t>>,
    pub span: Span<'t>,
}

impl Sentence<'_> {
    pub fn push(&mut self, s: impl Into<Self>) {
        for w in s.into().exprs {
            self.exprs.push(w)
        }
    }
}

impl<'t> Sentence<'t> {
    fn from_pair(p: pest::iterators::Pair<'t, Rule>) -> Self {
        assert_eq!(p.as_rule(), Rule::sentence);
        let span = p.as_span();

        let mut inner = p.into_inner();

        let first = inner.next().unwrap();

        let (args, body) = if first.as_rule() == Rule::sentence_args {
            let args = first
                .into_inner()
                .map(|i| ident_from_pair(i).to_owned())
                .collect();
            (Some(args), inner.exactly_one().unwrap())
        } else {
            (None, first)
        };

        assert_eq!(body.as_rule(), Rule::sentence_body);
        let exprs = body.into_inner().map(Expression::from_pair).collect();
        Sentence { span, args, exprs }
    }
}
