macro_rules! phrase {
    (add) => {
        Word::Add
    };
    (curry) => {
        Word::Curry
    };
    (copy($idx:expr)) => {
        Word::Copy($idx)
    };
    (drop($idx:expr)) => {
        Word::Drop($idx)
    };
    (mv($idx:expr)) => {
        Word::Move($idx)
    };
    (* $name:ident) => {
        Word::Push(Value::Symbol(stringify!($name)))
    };
    (# $val:expr) => {
        Word::Push(Value::from($val))
    };
    ($name:ident) => {
        Word::Push(Value::Reference(stringify!($name).to_string()))
    };
}

macro_rules! value {
    (@phrasecat ($($phrase:tt)*) ($($tail:tt)*) ) => {
        {
            let mut res :Sentence= Sentence(vec![]);
            res.push(phrase!($($phrase)*));
            res.push(value!(@sent ($($tail)*)));
            res
        }
    };
    (@sent ()) => { Sentence(vec![]) };

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
        Code::Sentence(
            value!(@sent ($($a)*)),
        )
    };
    (@code ($($a:tt)*) (; $($tail:tt)*)) => {
        Code::AndThen(
            value!(@sent ($($a)*)),
            Box::new(value!(@code () ($($tail)*)))
        )
    };
    (@code ($($a:tt)*) ($head:tt $($tail:tt)*)) => {
        value!(@code ($($a)* $head) ($($tail)*))
    };
    ($i:ident) => {
        Value::Reference(stringify!($i))
    };
    ({$($code:tt)*}) => {
        Value::Quote(Box::new(value!(@code () ($($code)*))))
    };
    ($e:expr) => {
        Value::from($e)
    };
}

macro_rules! lib {
    (@lib () ()) => {
        Library {
            decls: vec![],
        }
    };
    (@lib (let $name:ident = $val:tt;) ($($tail:tt)*)) => {
        {
            let mut lib = lib!($($tail)*);
            lib.decls.insert(0, Decl {
                name: stringify!($name).to_string(),
                value: value!($val),
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
