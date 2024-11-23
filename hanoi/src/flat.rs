use std::{collections::VecDeque, usize};

use derive_more::derive::{From, Into};
use itertools::Itertools;
use pest::{iterators::Pair, Span};
use typed_index_collections::TiVec;

use crate::ast::{
    self, ident_from_pair, literal_from_pair, Expression, InnerExpression, Path, Rule,
};

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
        res.visit_ns(lib, None);
        res
    }

    pub fn root_namespace(&self) -> &Namespace {
        self.namespaces.first().unwrap()
    }

    fn visit_ns(
        &mut self,
        ns: ast::Namespace<'t>,
        parent: Option<NamespaceIndex>,
    ) -> NamespaceIndex {
        let ns_idx = self.namespaces.push_and_get_key(Namespace::default());

        if let Some(parent) = parent {
            self.namespaces[ns_idx]
                .0
                .push(("super".to_owned(), Entry::Namespace(parent)));
        }

        for decl in ns.decls {
            match decl.value {
                ast::DeclValue::Namespace(namespace) => {
                    let subns = self.visit_ns(namespace, Some(ns_idx));
                    self.namespaces[ns_idx]
                        .0
                        .push((decl.name, Entry::Namespace(subns)));
                }
                ast::DeclValue::Code(code) => {
                    let sentence_idx = self.visit_code(&decl.name, ns_idx, VecDeque::new(), code);
                    self.namespaces[ns_idx].0.push((
                        decl.name,
                        Entry::Value(Value::Pointer(Closure(vec![], sentence_idx))),
                    ));
                }
                ast::DeclValue::Proc(ast::Proc { args, body }) => {
                    let sentence_idx = self.visit_proc(&decl.name, ns_idx, args, body);
                    self.namespaces[ns_idx].0.push((
                        decl.name,
                        Entry::Value(Value::Pointer(Closure(vec![], sentence_idx))),
                    ));
                }
            }
        }
        ns_idx
    }

    fn visit_proc(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        args: Pair<'t, Rule>,
        body: Pair<'t, Rule>,
    ) -> SentenceIndex {
        let mut names: VecDeque<Option<String>> = args
            .into_inner()
            .map(|p| ident_from_pair(p).as_str().to_owned())
            // .chain(["caller".to_owned()])
            .map(Some)
            .collect();
        self.visit_proc_block_pair(name, ns_idx, names, body)
    }

    fn visit_proc_block_pair(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        mut names: VecDeque<Option<String>>,
        body: Pair<'t, Rule>,
    ) -> SentenceIndex {
        let (statements, endpoint) = body.into_inner().collect_tuple().unwrap();

        assert_eq!(statements.as_rule(), Rule::proc_statements);
        let statements = statements.into_inner().collect();
        self.visit_proc_block(name, ns_idx, names, statements, endpoint)
    }

    fn visit_proc_block(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        mut names: VecDeque<Option<String>>,
        mut statements: VecDeque<Pair<'t, Rule>>,
        endpoint: Pair<'t, Rule>,
    ) -> SentenceIndex {
        let Some(statement) = statements.pop_front() else {
            return self.visit_proc_endpoint(name, ns_idx, names, endpoint);
        };

        assert_eq!(statement.as_rule(), Rule::proc_statement);
        let statement = statement.into_inner().exactly_one().unwrap();
        let span = statement.as_span();
        match statement.as_rule() {
            Rule::proc_let => {
                let (let_names, expr) = statement.into_inner().collect_tuple().unwrap();
                let let_names = let_names
                    .into_inner()
                    .map(|p| Some(ident_from_pair(p).as_str().to_owned()))
                    .collect_vec();

                let mut builder =
                    SentenceBuilder::new(Some(name.to_owned()), ns_idx, names.clone());

                builder.proc_expr(expr);

                builder.sd_top(span);

                let mut next_names = builder.names.clone();
                next_names.pop_back();
                next_names.extend(let_names);

                let next = self.visit_proc_block(name, ns_idx, next_names, statements, endpoint);

                builder.literal(span, Value::Pointer(Closure(vec![], next)));
                while builder.names.len() > 2 {
                    builder.builtin(span, Builtin::Curry);
                }
                builder.mv_idx(span, 1);
                builder.builtin(span, Builtin::Curry);
                builder.literal(span, Value::Symbol("exec".to_owned()));

                self.sentences.push_and_get_key(builder.build())
            }
            _ => unreachable!("Unexpected rule: {:?}", statement),
        }
    }

    fn visit_proc_endpoint(
        &mut self,
        name: &str,
        ns_idx: NamespaceIndex,
        names: VecDeque<Option<String>>,
        endpoint: Pair<'t, Rule>,
    ) -> SentenceIndex {
        assert_eq!(endpoint.as_rule(), Rule::proc_endpoint);
        let endpoint = endpoint.into_inner().exactly_one().unwrap();

        match endpoint.as_rule() {
            Rule::proc_func_call => {
                let span = endpoint.as_span();

                let mut builder =
                    SentenceBuilder::new(Some(name.to_owned()), ns_idx, names.clone());

                builder.proc_func_call(endpoint);

                while builder.names.len() > 1 {
                    builder.drop_idx(span, 1);
                }
                builder.literal(span, Value::Symbol("exec".to_owned()));

                self.sentences.push_and_get_key(builder.build())
            }
            Rule::proc_if => {
                let span = endpoint.as_span();
                let (cond, true_case, false_case) = endpoint.into_inner().collect_tuple().unwrap();

                let cond = ident_from_pair(cond);

                let mut builder =
                    SentenceBuilder::new(Some(name.to_owned()), ns_idx, names.clone());
                builder.mv(cond, cond.as_str());

                let mut case_names = builder.names.clone();
                case_names.pop_front();

                let true_case =
                    self.visit_proc_block_pair(name, ns_idx, case_names.clone(), true_case);
                let false_case = self.visit_proc_block_pair(name, ns_idx, case_names, false_case);

                builder.literal(span, Value::Pointer(Closure(vec![], true_case)));
                builder.literal(span, Value::Pointer(Closure(vec![], false_case)));
                builder.literal(span, Value::Symbol("if".to_owned()));

                self.sentences.push_and_get_key(builder.build())
            }
            Rule::proc_match_block => {
                let span = endpoint.as_span();
                let (expr, cases) = endpoint.into_inner().collect_tuple().unwrap();

                let mut builder =
                    SentenceBuilder::new(Some(name.to_owned()), ns_idx, names.clone());

                builder.proc_expr(expr);
                // Stack: (leftover names) to_call

                builder.sd_top(span);
                // Stack: to_call (leftover names)

                let mut leftover_names = builder.names.clone();
                leftover_names.pop_back();
                dbg!(&leftover_names);

                assert_eq!(cases.as_rule(), Rule::proc_match_cases);
                let cases = cases.into_inner().collect_vec();

                let mut panic_builder =
                    SentenceBuilder::new(Some(name.to_owned()), ns_idx, VecDeque::new());
                panic_builder.symbol(span, "panic");
                let panic_idx = self.sentences.push_and_get_key(panic_builder.build());

                let mut next_case = panic_idx;
                for case in cases.into_iter().rev() {
                    let case_span = case.as_span();
                    let (discrim, bindings, body) = case.into_inner().collect_tuple().unwrap();

                    let if_case_matches_names :VecDeque<Option<String>> =
                        // Preserved names from before the call.
                        leftover_names.iter().cloned()
                        // Empty slot for the discriminator.
                        .chain([None])
                        // Then the bindings.
                        .chain(
                            bindings
                            .into_inner()
                            .map(|p| Some(ident_from_pair(p).as_str().to_owned()))
                        ).collect();

                    let if_case_matches_idx =
                        self.visit_proc_block_pair(name, ns_idx, if_case_matches_names, body);

                    let discrim = literal_from_pair(discrim);

                    let mut case_builder =
                        SentenceBuilder::new(Some(name.to_owned()), ns_idx, VecDeque::new());
                    // Copy the first thing after the leftover names.
                    case_builder.cp_idx(case_span, leftover_names.len());
                    case_builder.literal(case_span, discrim);
                    case_builder.builtin(case_span, Builtin::Eq);
                    case_builder.literal(
                        case_span,
                        Value::Pointer(Closure(vec![], if_case_matches_idx)),
                    );
                    case_builder.literal(case_span, Value::Pointer(Closure(vec![], next_case)));
                    case_builder.symbol(case_span, "if");
                    next_case = self.sentences.push_and_get_key(case_builder.build());
                }

                builder.literal(span, Value::Pointer(Closure(vec![], next_case)));
                // Stack: to_call (leftover names) match_beginning

                while builder.names.len() > 2 {
                    builder.builtin(span, Builtin::Curry);
                }
                // Stack: to_call curried_match_beginning
                builder.mv_idx(span, 1);
                builder.builtin(span, Builtin::Curry);
                builder.symbol(span, "exec");

                self.sentences.push_and_get_key(builder.build())
            }
            _ => unreachable!("Unexpected rule: {:?}", endpoint),
        }
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
                        InnerWord::Push(Value::Pointer(Closure(vec![], and_then))).into(),
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
                            Value::Pointer(Closure(vec![], true_case)).into(),
                            Value::Pointer(Closure(vec![], false_case)).into(),
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
                            Value::Pointer(Closure(vec![], body)).into(),
                            Value::Pointer(Closure(vec![], next_case)).into(),
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
        let mut stash_names: Vec<Option<String>> = vec![];

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
                    InnerWord::Ref(idx) => names.push_front(None),
                    InnerWord::Builtin(builtin) => match builtin {
                        Builtin::Add
                        | Builtin::Eq
                        | Builtin::Curry
                        | Builtin::Or
                        | Builtin::And
                        | Builtin::Get
                        | Builtin::SymbolCharAt
                        | Builtin::Cons => {
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
                        Builtin::Not | Builtin::SymbolLen | Builtin::Deref => {
                            names.pop_front();
                            names.push_front(None);
                        }
                        Builtin::AssertEq => {
                            names.pop_front();
                            names.pop_front();
                        }
                        Builtin::Snoc => {
                            names.pop_front();
                            names.push_front(None);
                            names.push_front(None);
                        }
                        Builtin::Stash => {
                            stash_names.push(names.pop_front().unwrap());
                        }
                        Builtin::Unstash => {
                            names.push_front(stash_names.pop().unwrap());
                        }
                    },
                }
                words.push(w)
            }
        }
        assert_eq!(stash_names, vec![]);
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
                "ref" => InnerWord::Ref(idx),
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

pub struct SentenceBuilder<'t> {
    pub name: Option<String>,
    pub ns_idx: NamespaceIndex,
    pub names: VecDeque<Option<String>>,
    pub words: Vec<Word<'t>>,
}

impl<'t> SentenceBuilder<'t> {
    pub fn new(
        name: Option<String>,
        ns_idx: NamespaceIndex,
        names: VecDeque<Option<String>>,
    ) -> Self {
        Self {
            name,
            ns_idx,
            names,
            words: vec![],
        }
    }

    pub fn build(self) -> Sentence<'t> {
        Sentence {
            name: self.name,
            words: self.words,
        }
    }

    pub fn literal(&mut self, span: Span<'t>, value: Value) {
        self.words.push(Word {
            inner: InnerWord::Push(value),
            span: Some(span),
            names: Some(self.names.clone()),
        });
        self.names.push_front(None);
    }

    pub fn symbol(&mut self, span: Span<'t>, symbol: &str) {
        self.literal(span, Value::Symbol(symbol.to_owned()))
    }

    pub fn mv(&mut self, span: Span<'t>, name: &str) {
        let Some(idx) = self.names.iter().position(|n| match n {
            Some(n) => n.as_str() == name,
            None => false,
        }) else {
            panic!("unknown reference: {:?}", name)
        };
        self.mv_idx(span, idx)
    }

    pub fn mv_idx(&mut self, span: Span<'t>, idx: usize) {
        let names = self.names.clone();
        let declared = self.names.remove(idx).unwrap();
        self.names.push_front(declared);

        self.words.push(Word {
            inner: InnerWord::Move(idx),
            span: Some(span),
            names: Some(names),
        });
    }

    pub fn cp(&mut self, span: Span<'t>, name: &str) {
        let Some(idx) = self.names.iter().position(|n| match n {
            Some(n) => n.as_str() == name,
            None => false,
        }) else {
            panic!("unknown reference: {:?}", name)
        };
        self.cp_idx(span, idx)
    }

    pub fn cp_idx(&mut self, span: Span<'t>, idx: usize) {
        let names = self.names.clone();
        self.names.push_front(None);

        self.words.push(Word {
            inner: InnerWord::Copy(idx),
            span: Some(span),
            names: Some(names),
        });
    }

    pub fn sd_idx(&mut self, span: Span<'t>, idx: usize) {
        let names = self.names.clone();

        let declared = self.names.pop_front().unwrap();
        self.names.insert(idx, declared);

        self.words.push(Word {
            inner: InnerWord::Send(idx),
            span: Some(span),
            names: Some(names),
        });
    }
    pub fn sd_top(&mut self, span: Span<'t>) {
        self.sd_idx(span, self.names.len() - 1)
    }

    pub fn drop_idx(&mut self, span: Span<'t>, idx: usize) {
        let names = self.names.clone();
        let declared = self.names.remove(idx).unwrap();

        self.words.push(Word {
            inner: InnerWord::Drop(idx),
            span: Some(span),
            names: Some(names),
        });
    }

    pub fn path(&mut self, Path { span, segments }: Path<'t>) {
        for segment in segments.iter() {
            self.literal(*segment, Value::Symbol(segment.as_str().to_owned()));
        }
        self.literal(span, Value::Namespace(self.ns_idx));
        for segment in segments {
            self.builtin(segment, Builtin::Get);
        }
    }

    pub fn builtin(&mut self, span: Span<'t>, builtin: Builtin) {
        self.words.push(Word {
            inner: InnerWord::Builtin(builtin),
            span: Some(span),
            names: Some(self.names.clone()),
        });
        match builtin {
            Builtin::Add
            | Builtin::Eq
            | Builtin::Curry
            | Builtin::Or
            | Builtin::And
            | Builtin::Get
            | Builtin::SymbolCharAt
            | Builtin::Cons => {
                self.names.pop_front();
                self.names.pop_front();
                self.names.push_front(None);
            }
            Builtin::NsEmpty => {
                self.names.push_front(None);
            }
            Builtin::NsGet => {
                let ns = self.names.pop_front().unwrap();
                self.names.pop_front();
                self.names.push_front(ns);
                self.names.push_front(None);
            }
            Builtin::NsInsert => {
                self.names.pop_front();
                self.names.pop_front();
                self.names.pop_front();
                self.names.push_front(None);
            }
            Builtin::NsRemove => {
                let ns = self.names.pop_front().unwrap();
                self.names.pop_front();
                self.names.push_front(ns);
                self.names.push_front(None);
            }
            Builtin::Not | Builtin::SymbolLen | Builtin::Deref => {
                self.names.pop_front();
                self.names.push_front(None);
            }
            Builtin::AssertEq => {
                self.names.pop_front();
                self.names.pop_front();
            }
            Builtin::Snoc => {
                self.names.pop_front();
                self.names.push_front(None);
                self.names.push_front(None);
            }
            Builtin::Stash => {
                todo!()
            }
            Builtin::Unstash => {
                todo!()
            }
        }
    }

    fn proc_expr(&mut self, expr: Pair<'t, Rule>) {
        assert_eq!(expr.as_rule(), Rule::proc_expr);
        let expr = expr.into_inner().exactly_one().unwrap();
        match expr.as_rule() {
            Rule::proc_func_call => {
                self.proc_func_call(expr);
            }
            _ => unreachable!("Unexpected rule: {:?}", expr),
        }
    }

    fn proc_func_call(&mut self, func_call: Pair<'t, Rule>) {
        assert_eq!(func_call.as_rule(), Rule::proc_func_call);
        let (func, args) = func_call.into_inner().collect_tuple().unwrap();
        let func = ast::PathOrIdent::from(func);

        let args = args.into_inner().collect_vec();

        for arg in args.iter().rev() {
            match arg.as_rule() {
                Rule::identifier => {
                    let arg = ident_from_pair(arg.clone());
                    self.mv(arg, arg.as_str());
                }
                Rule::copy => {
                    let arg = arg.clone().into_inner().exactly_one().unwrap();
                    let arg = ident_from_pair(arg);
                    self.cp(arg, arg.as_str());
                }
                Rule::literal => {
                    let lit = literal_from_pair(arg.clone());
                    self.literal(arg.as_span(), lit);
                }
                _ => unreachable!("Unexpected rule: {:?}", arg),
            }
        }

        match func {
            ast::PathOrIdent::Path(p) => self.path(p),
            ast::PathOrIdent::Ident(i) => self.mv(i, i.as_str()),
        }

        for arg in args.into_iter() {
            self.builtin(arg.as_span(), Builtin::Curry);
        }
    }
}

builtins! {
    (Add, add),
    (Eq, eq),
    (AssertEq, assert_eq),
    (Curry, curry),
    (Or, or),
    (And, and),
    (Not, not),
    (Get, get),
    (SymbolCharAt, symbol_char_at),
    (SymbolLen, symbol_len),

    (NsEmpty, ns_empty),
    (NsInsert, ns_insert),
    (NsGet, ns_get),
    (NsRemove, ns_remove),

    (Cons, cons),
    (Snoc, snoc),

    (Deref, deref),

    (Stash, stash),
    (Unstash, unstash),
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
    Ref(usize),
    Builtin(Builtin),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Symbol(String),
    Usize(usize),
    List(Vec<Value>),
    Pointer(Closure),
    Handle(usize),
    Bool(bool),
    Char(char),
    Namespace(NamespaceIndex),
    Namespace2(Namespace2),
    Nil,
    Cons(Box<Value>, Box<Value>),
    Ref(usize),
}

impl Value {
    pub fn is_small(&self) -> bool {
        match self {
            Value::Nil
            | Value::Symbol(_)
            | Value::Usize(_)
            | Value::Char(_)
            | Value::Bool(_)
            | Value::Ref(_) => true,
            Value::Namespace(namespace_index) => todo!(),
            Value::Pointer(Closure(vec, _)) => true,
            Value::List(_) | Value::Handle(_) | Value::Namespace2(_) | Value::Cons(_, _) => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Closure(pub Vec<Value>, pub SentenceIndex);

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
            Value::Nil => write!(f, "nil"),
            Value::Cons(car, cdr) => write!(
                f,
                "cons({}, {})",
                ValueView {
                    lib: self.lib,
                    value: car
                },
                ValueView {
                    lib: self.lib,
                    value: cdr
                }
            ),
            Value::Ref(arg0) => write!(f, "ref({})", arg0),
            Value::Char(arg0) => write!(f, "'{}'", arg0),
            Value::Pointer(Closure(values, ptr)) => {
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
