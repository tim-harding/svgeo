use std::fmt::Display;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Str(&'static str),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(ValueVec),
    Object(ValueObj),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Str(s) => write!(f, "\"{s}\""),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Array(a) => {
                let a: Vec<_> = a.0.iter().map(|a| a.to_string()).collect();
                let s = a.join(",\n");
                write!(f, "[\n{s}\n]")
            }
            Value::Object(o) => {
                let a: Vec<_> =
                    o.0.iter()
                        .map(|(k, v)| {
                            let v = v.to_string();
                            format!("\"{k}\":{v}")
                        })
                        .collect();
                let s = a.join(",\n");
                write!(f, "{{\n{s}\n}}")
            }
        }
    }
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
value_from!(i64, Integer);
value_from!(f64, Float);
value_from!(bool, Boolean);
value_from!(ValueVec, Array);
value_from!(ValueObj, Object);

macro_rules! as_value {
    ($t:ty, $d:ident) => {
        impl From<$t> for Value {
            fn from(value: $t) -> Self {
                (value as $d).into()
            }
        }
    };
}

as_value!(f32, f64);
as_value!(isize, i64);
as_value!(i32, i64);
as_value!(i16, i64);
as_value!(i8, i64);
as_value!(usize, i64);
as_value!(u64, i64);
as_value!(u32, i64);
as_value!(u16, i64);
as_value!(u8, i64);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValueVec(pub Vec<Value>);

impl ValueVec {
    pub const fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, value: impl Into<Value>) {
        self.0.push(value.into());
    }
}

impl From<Vec<Value>> for ValueVec {
    fn from(value: Vec<Value>) -> Self {
        Self(value)
    }
}

#[macro_export]
macro_rules! value_vec {
    () => {
        $crate::json::ValueVec::new()
    };
    ($($e:expr),*) => {
        {
            let mut out = $crate::json::ValueVec::new();
            $(out.push($e);)*
            out
        }
    };
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValueObj(pub Vec<(&'static str, Value)>);

impl ValueObj {
    pub const fn new() -> Self {
        Self(vec![])
    }

    pub fn insert(&mut self, key: &'static str, value: impl Into<Value>) {
        self.0.push((key, value.into()));
    }
}

#[macro_export]
macro_rules! value_obj {
    () => {
        $crate::json::ValueObj::new()
    };
    ($($k:expr; $v:expr),*) => {
        {
            let mut out = $crate::json::ValueObj::new();
            $(out.insert($k, $v);)*
            out
        }
    };
}

impl From<Vec<(&'static str, Value)>> for ValueObj {
    fn from(value: Vec<(&'static str, Value)>) -> Self {
        Self(value)
    }
}
