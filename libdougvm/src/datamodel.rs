use std::any::Any;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::bytecode::Op;

pub type Integer = i64;
pub type Real = f64;
pub type Unknown = Rc<dyn Any>;

pub type NativeFn = fn(Vec<Value>) -> Value;

#[derive(Clone)]
pub struct Buffer {
    items: Rc<RefCell<Vec<u8>>>,
}

#[derive(Clone)]
pub struct Function {
    pub module: Tuple,
    pub ops: Rc<[Op]>,
}

#[derive(Clone)]
pub struct List {
    items: Rc<RefCell<Vec<Value>>>,
}

#[derive(Clone)]
pub struct Table {
    items: Rc<RefCell<Vec<(u64, Value)>>>,
}

#[derive(Clone)]
pub struct Tuple {
    items: Rc<[RefCell<Value>]>,
}

#[derive(Clone)]
pub struct TupleWeak {
    weakref: Weak<[RefCell<Value>]>,
}

macro_rules! create_value_enum {
    ($($n:ident),+) => {
        #[derive(Clone)]
        pub enum Value {
            None,
            $($n($n)),+
        }

        impl Value {
            pub fn get_type(&self) -> ValueType {
                match self {
                    Value::None => ValueType::None,
                    $(Value::$n(_) => ValueType::$n),+
                }
            }
        }

        #[repr(u8)]
        #[derive(PartialEq)]
        pub enum ValueType {
            None,
            $($n),+
        }

        impl ValueType {
            pub fn as_str(&self) -> &'static str {
                match self {
                    ValueType::None => "None",
                    $(ValueType::$n => stringify!($n)),+
                }
            }
        }

        $(create_value_enum!(conversion $n);)+
    };
    (conversion $t:ident) => {
        impl From< $t > for Value {
            fn from(t: $t ) -> Self {
                Value:: $t (t)
            }
        }

        impl TryInto< $t > for Value {
            type Error = ValueTryIntoError;
            fn try_into(self) -> Result<$t, Self::Error> {
                match self {
                    Value :: $t (t) => Ok(t),
                    _ => Err(ValueTryIntoError {
                        found: self.get_type(),
                        expected: ValueType::$t,
                    }),
                }
            }
        }

        impl<'a> TryInto< &'a $t > for &'a Value {
            type Error = ValueTryIntoError;
            fn try_into(self) -> Result<&'a $t, Self::Error> {
                match self {
                    Value :: $t (t) => Ok(t),
                    _ => Err(ValueTryIntoError {
                        found: self.get_type(),
                        expected: ValueType::$t,
                    }),
                }
            }
        }
    };
}

create_value_enum! {
    Integer, Real, Tuple, TupleWeak, Table, List, Buffer, Function, NativeFn, Unknown
}

pub struct ValueTryIntoError {
    pub found: ValueType,
    pub expected: ValueType,
}

pub trait Identity {
    fn identity(&self) -> usize;
}

impl Identity for Unknown {
    fn identity(&self) -> usize {
        Rc::as_ptr(self).cast::<()>() as usize
    }
}

impl From<bool> for Value {
    fn from(t: bool) -> Self {
        match t {
            true => 1.into(),
            false => 0.into(),
        }
    }
}

impl From<Ordering> for Value {
    fn from(t: Ordering) -> Self {
        Value::Integer(match t {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        })
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(t: Option<T>) -> Self {
        match t {
            Some(t) => t.into(),
            None => Value::None,
        }
    }
}
