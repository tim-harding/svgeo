use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum Value {
    String(String),
    Str(&'static str),
    Integer(u64),
    Float(f64),
    Boolean(bool),
    Array(ValueVec),
    Object(ValueObj),
}

macro_rules! value_from {
    ($t:ty, $i:ident) => {
        impl From<$t> for Value {
            fn from(value: $t) -> Self {
                Value::$i(value)
            }
        }
    };
}

value_from!(String, String);
value_from!(&'static str, Str);
value_from!(u64, Integer);
value_from!(f64, Float);
value_from!(bool, Boolean);
value_from!(ValueVec, Array);
value_from!(ValueObj, Object);

#[derive(Clone, Debug, Default, PartialEq)]
struct ValueVec(Vec<Value>);

impl ValueVec {
    pub const fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        self.0.push(value.into());
    }
}

macro_rules! value_vec {
    ($($e:expr),*) => {
        {
            let mut out = ValueVec::new();
            $(out.push($e);)*
            out
        }
    };
}

#[derive(Clone, Debug, Default, PartialEq)]
struct ValueObj(Vec<(&'static str, Value)>);

impl ValueObj {
    pub const fn new() -> Self {
        Self(vec![])
    }

    pub fn insert(&mut self, key: &'static str, value: Value) {
        self.0.push((key, value));
    }
}

macro_rules! value_obj {
    ($($k:expr; $v:expr),*) => {
        {
            let mut out = ValueObj::new();
            $(out.push($k, $v);)*
            out
        }
    };
}

fn structure() -> Value {
    value_vec![
        "fileversion",
        "20.5.332",
        "hasindex",
        false,
        "pointcount",
        52,
        "vertexcount",
        52,
        "primitivecount",
        2,
        "info",
        value_obj!()
    ]
    .into()
}
