macro_rules! phrase {
    ($name:ident($idx:expr)) => {
        ast::Expression::FunctionLike(stringify!($name), $idx)
    };
    (drop($idx:expr)) => {
        Word::Drop($idx)
    };
    (mv($idx:expr)) => {
        Word::Move($idx)
    };
    (* $name:ident) => {
        ast::Expression::Symbol(stringify!($name))
    };
    (# $val:expr) => {
        ast::Expression::from($val)
    };
    ($name:ident) => {
        ast::Expression::Reference(stringify!($name))
    };
}

macro_rules! value {
    (@phrasecat ($($phrase:tt)*) ($($tail:tt)*) ) => {
        ast::Sentence(std::iter::once(phrase!($($phrase)*))
            .chain(value!(@sent ($($tail)*)).0.into_iter())
            .collect())
    };
    (@sent ()) => { ast::Sentence(vec![]) };

    (@sent (* $symbol:ident $($tail:tt)*)) => {
        value!(@phrasecat (* $symbol) ($($tail)*))
    };
    (@sent (# $val:tt $($tail:tt)*)) => {
        value!(@phrasecat (# $val) ($($tail)*))
    };
    (@sent ($flike:ident($($head:tt)*) $($tail:tt)*)) => {
        value!(@phrasecat ($flike($($head)*)) ($($tail)*))
    };
    (@sent ($head:tt $($tail:tt)*)) => {
        value!(@phrasecat ($head) ($($tail)*))
    };
    (@code ($($a:tt)*) ()) => {
        ast::Code::Sentence(
            value!(@sent ($($a)*)),
        )
    };
    (@code ($($a:tt)*) (; $($tail:tt)*)) => {
        ast::Code::AndThen(
            value!(@sent ($($a)*)),
            Box::new(value!(@code () ($($tail)*)))
        )
    };
    (@code ($($a:tt)*) (if { $($true:tt)* } else { $($false:tt)* })) => {
        ast::Code::If{
            cond: value!(@sent ($($a)*)),
            true_case: Box::new(value!(@code () ($($true)*))),
            false_case: Box::new(value!(@code () ($($false)*))),
        }
    };
    (@code ($($a:tt)*) ($head:tt $($tail:tt)*)) => {
        value!(@code ($($a)* $head) ($($tail)*))
    };
    ($i:ident) => {
        Value::Reference(stringify!($i))
    };
    ($e:expr) => {
        Value::from($e)
    };
}

macro_rules! lib {
    (@lib () ()) => {
        ast::Library {
            decls: vec![],
        }
    };
    (@lib (let $name:ident = {$($code:tt)*};) ($($tail:tt)*)) => {
        {
            let mut lib = lib!($($tail)*);
            lib.decls.insert(0, ast::Decl {
                name: stringify!($name).to_string(),
                value: value!(@code () ($($code)*)),
            });
            lib
        }
    };
    (@lib ($($a:tt)*) ($head:tt $($tail:tt)*)) => {
        lib!(@lib ($($a)* $head) ($($tail)*))
    };
    ($($tail:tt)*) => {
        lib!(@lib () ($($tail)*))
    };
}
