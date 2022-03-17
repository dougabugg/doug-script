use std::mem::swap;

pub mod bytecode;
pub mod datamodel;

use crate::bytecode::{OpAction, OpError, Operation};
use crate::datamodel::{Function, Value};

pub struct CallFrame {
    pub parent: Option<Box<CallFrame>>,
    pub function: Function,
    pub cursor: usize,
    pub stack: CallStack,
}

impl CallFrame {
    pub fn new(function: Function) -> CallFrame {
        let mut stack = CallStack::new();
        stack.store(0, function.module.clone().into());
        CallFrame {
            parent: None,
            function,
            cursor: 0,
            stack,
        }
    }

    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    pub fn jump(&mut self, index: i32) {
        self.cursor = (self.cursor as isize + index as isize) as usize;
    }

    pub fn exec(&mut self) -> Result<OpAction, OpError> {
        let op = match self.function.ops.get(self.cursor) {
            Some(op) => op.clone(),
            None => return Ok(OpAction::Return(Value::None)),
        };
        self.cursor += 1;
        op.exec(&mut self.stack)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub struct CallStack {
    stack: Vec<Value>,
    locals: Vec<Value>,
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack {
            stack: Vec::new(),
            locals: Vec::new(),
        }
    }

    pub fn load(&self, index: u8) -> Result<&Value, OpError> {
        self.locals
            .get(index as usize)
            .ok_or(OpError::LocalRead(index))
    }

    fn get_mut_or_resize(&mut self, index: u8) -> &mut Value {
        let index = index as usize;
        if index >= self.locals.len() {
            self.locals.resize_with(index + 1, || Value::None);
        }
        unsafe { self.locals.get_unchecked_mut(index) }
    }

    pub fn store(&mut self, index: u8, val: Value) {
        let out = self.get_mut_or_resize(index);
        *out = val;
    }

    pub fn swap(&mut self, index: u8, val: &mut Value) {
        let out = self.get_mut_or_resize(index);
        swap(out, val);
    }

    pub fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    pub fn pop(&mut self) -> Result<Value, OpError> {
        self.stack.pop().ok_or(OpError::StackEmpty)
    }
}

pub struct VirtualMachine {
    frame: Option<Box<CallFrame>>,
}

impl VirtualMachine {
    pub fn new(func: Function) -> VirtualMachine {
        VirtualMachine {
            frame: Some(Box::new(CallFrame::new(func))),
        }
    }

    pub fn run_until_exited(&mut self) -> Result<Value, OpError> {
        loop {
            let action = self.step()?;
            match self.process(action)? {
                VmState::Running => continue,
                VmState::Exited(val) => return Ok(val),
            }
        }
    }

    pub fn step(&mut self) -> Result<OpAction, OpError> {
        let frame = self.frame.as_mut().unwrap();
        frame.exec()
    }

    pub fn process(&mut self, action: OpAction) -> Result<VmState, OpError> {
        match action {
            OpAction::None => (),
            OpAction::Jump(dest) => {
                let frame = self.frame.as_mut().unwrap();
                frame.jump(dest);
            }
            OpAction::Call(func, args) => {
                let mut callee = Box::new(CallFrame::new(func));
                // NOTE: for expr `Call(A, B, C)`, args is reversed: `[C, B, A]`
                // so now the order that they will be popped off the stack is
                // (A, B, C), which is how the stage0 compiler expects them.
                // see crate::bytecode::ops::Call for details
                for arg in args.into_iter() {
                    callee.push(arg);
                }
                swap(&mut self.frame, &mut callee.parent);
                self.frame = Some(callee);
            }
            OpAction::CallNative(func, args) => {
                let frame = self.frame.as_mut().unwrap();
                frame.push(func(args));
            }
            OpAction::Return(val) => {
                let frame = self.frame.as_mut().unwrap();
                let mut parent = None;
                swap(&mut frame.parent, &mut parent);
                match parent {
                    Some(mut parent) => {
                        parent.push(val);
                        self.frame = Some(parent);
                    }
                    None => {
                        self.frame = None;
                        return Ok(VmState::Exited(val));
                    }
                }
            }
        }
        Ok(VmState::Running)
    }
}

pub enum VmState {
    Running,
    Exited(Value),
}
