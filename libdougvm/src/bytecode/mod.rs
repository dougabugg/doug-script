use crate::CallStack;
use crate::datamodel::{Value, Function, NativeFn, ValueType, ValueTryIntoError};

pub trait Operation {
    fn exec(&self, m: &mut CallStack) -> Result<OpAction, OpError>;
}

pub enum Op {}

impl Operation for Op {
    fn exec(&self, m: &mut CallStack) -> Result<OpAction, OpError> {
        panic!()
    }
}

pub enum OpAction {
    None,
    Jump(i32),
    Call(Function, Vec<Value>),
    CallNative(NativeFn, Vec<Value>),
    Return(Value),
}

pub enum OpError {
    StackEmpty,
    LocalRead(u8),
    IndexRead(i64),
    IndexWrite(i64),
    IntoType(ValueTryIntoError),
    BadType(ValueType),
}
