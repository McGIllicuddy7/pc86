use std::{
    error::Error,
    ops::Not,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub enum Instruction {
    Move {
        to: Operand,
        from: Operand,
    },
    BinaryOperator {
        op: Operator,
        left: Operand,
        right: Operand,
        output: Operand,
    },
    Not {
        input: Operand,
        output: Operand,
    },
    BeginStackFrame {
        variable_count: usize,
    },
    Return {
        to_return: Variable,
    },
    Call {
        to_call: usize,
        arguments: Arc<[Operand]>,
        output: Option<Operand>,
    },
    Jmp {
        to_jump_to: usize,
    },
    JmpIfNot {
        to_jump_to: usize,
        condition: Operand,
    },
    Halt,
    Write {
        to_write: Operand,
    },
    WriteLn {
        to_write: Operand,
    },
    Read {
        to_write_to: Operand,
    },
}

#[derive(Debug, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    CmpGe,
    CmpNe,
    CmpLe,
    CmpE,
    CmpG,
    CmpL,
    And,
    Or,
    Xor,
}
#[derive(Debug, Clone)]
pub enum Operand {
    StackVariable { v: usize },
    FieldAccess { v: usize, of: usize },
    ArrayAccess { v: usize, of: Box<Operand> },
    ConstantUnsigned(u64),
    ConstantSigned(i64),
    ConstantFloat(f64),
    ConstantString(Arc<str>),
    NullPtr,
}
#[derive(Debug, Clone)]
pub enum Variable {
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Pointer(usize),
    String(Arc<str>),
}

#[derive(Debug, Clone)]
pub struct HeapValue {
    pub gc_is_reachable: bool,
    pub is_array: bool,
    pub field_names: Vec<Arc<str>>,
    pub fields: Vec<Variable>,
}

#[derive(Debug, Clone)]
pub struct ProgramData {
    pub heap: Arc<[Mutex<Option<HeapValue>>]>,
    pub instructions: Arc<[Instruction]>,
}

#[derive(Debug, Clone)]
pub struct Machine {
    pub data: ProgramData,
    pub instruction_pointer: usize,
    pub current_frame: Vec<Variable>,
    pub previous_frames: Vec<(usize, Vec<Variable>, Option<Operand>)>,
    pub halted: bool,
}
impl Machine {
    pub fn step(&mut self) -> Result<(), Box<dyn Error>> {
        let instruction = &self.data.instructions[self.instruction_pointer];
        self.instruction_pointer += 1;
        match instruction {
            Instruction::Move { to, from } => {
                let tmp = from.read(&self.data, &self.current_frame)?;
                to.write(tmp, &self.data, &mut self.current_frame)?;
            }
            Instruction::BinaryOperator {
                op,
                left,
                right,
                output,
            } => {
                let lft = left.read(&self.data, &self.current_frame)?;
                let rght = right.read(&self.data, &self.current_frame)?;
                let out;
                match lft {
                    Variable::Bool(l) => {
                        match rght {
                            Variable::Bool(r) => {
                                match op {
                                    Operator::And => {
                                        out = Variable::Bool(l && r);
                                    }
                                    Operator::Or => {
                                        out = Variable::Bool(l || r);
                                    }
                                    Operator::Xor => {
                                        out = Variable::Bool(l ^ r);
                                    }
                                    _ => {
                                        return Err(format!(
                                            "incompatible operation for types:{:#?} and {:#?} {:#?}",
                                            left, right, op
                                        ).into());
                                    }
                                }
                                _ = 0;
                            }
                            _ => {
                                return Err(format!(
                                    "incompatible types:{:#?} and {:#?}",
                                    left, right
                                )
                                .into());
                            }
                        }
                        _ = 0;
                    }
                    Variable::Unsigned(l) => {
                        match rght {
                            Variable::Unsigned(r) => {
                                match op {
                                    Operator::Add => {
                                        out = Variable::Unsigned(l.wrapping_add(r));
                                    }
                                    Operator::Sub => {
                                        out = Variable::Unsigned(l.wrapping_sub(r));
                                    }
                                    Operator::Mul => {
                                        out = Variable::Unsigned(l.wrapping_mul(r));
                                    }
                                    Operator::Div => {
                                        out = Variable::Unsigned(l.wrapping_div(r));
                                    }
                                    Operator::Rem => {
                                        out = Variable::Unsigned(l.wrapping_rem(r));
                                    }
                                    Operator::CmpGe => {
                                        out = Variable::Bool(l >= r);
                                    }
                                    Operator::CmpNe => {
                                        out = Variable::Bool(l != r);
                                    }
                                    Operator::CmpLe => {
                                        out = Variable::Bool(l <= r);
                                    }
                                    Operator::CmpE => {
                                        out = Variable::Bool(l == r);
                                    }
                                    Operator::CmpG => {
                                        out = Variable::Bool(l > r);
                                    }
                                    Operator::CmpL => {
                                        out = Variable::Bool(l < r);
                                    }
                                    Operator::And => {
                                        out = Variable::Unsigned(l & r);
                                    }
                                    Operator::Or => {
                                        out = Variable::Unsigned(l | r);
                                    }
                                    Operator::Xor => {
                                        out = Variable::Unsigned(l ^ r);
                                    }
                                }
                                _ = 0;
                            }
                            _ => {
                                return Err(format!(
                                    "incompatible types:{:#?} and {:#?}",
                                    left, right
                                )
                                .into());
                            }
                        }
                        _ = 0;
                    }
                    Variable::Signed(l) => {
                        match rght {
                            Variable::Signed(r) => {
                                match op {
                                    Operator::Add => {
                                        out = Variable::Signed(l.wrapping_add(r));
                                    }
                                    Operator::Sub => {
                                        out = Variable::Signed(l.wrapping_sub(r));
                                    }
                                    Operator::Mul => {
                                        out = Variable::Signed(l.wrapping_mul(r));
                                    }
                                    Operator::Div => {
                                        out = Variable::Signed(l.wrapping_div(r));
                                    }
                                    Operator::Rem => {
                                        out = Variable::Signed(l.wrapping_rem(r));
                                    }
                                    Operator::CmpGe => {
                                        out = Variable::Bool(l >= r);
                                    }
                                    Operator::CmpNe => {
                                        out = Variable::Bool(l != r);
                                    }
                                    Operator::CmpLe => {
                                        out = Variable::Bool(l <= r);
                                    }
                                    Operator::CmpE => {
                                        out = Variable::Bool(l == r);
                                    }
                                    Operator::CmpG => {
                                        out = Variable::Bool(l > r);
                                    }
                                    Operator::CmpL => {
                                        out = Variable::Bool(l < r);
                                    }
                                    Operator::And => {
                                        out = Variable::Signed(l & r);
                                    }
                                    Operator::Or => {
                                        out = Variable::Signed(l | r);
                                    }
                                    Operator::Xor => {
                                        out = Variable::Signed(l ^ r);
                                    }
                                }
                                _ = 0;
                            }
                            _ => {
                                return Err(format!(
                                    "incompatible types:{:#?} and {:#?}",
                                    left, right
                                )
                                .into());
                            }
                        }
                        _ = 0;
                    }
                    Variable::Float(l) => {
                        match rght {
                            Variable::Float(r) => {
                                match op {
                                    Operator::Add => {
                                        out = Variable::Float(l + r);
                                    }
                                    Operator::Sub => {
                                        out = Variable::Float(l - r);
                                    }
                                    Operator::Mul => {
                                        out = Variable::Float(l * r);
                                    }
                                    Operator::Div => {
                                        out = Variable::Float(l / r);
                                    }
                                    Operator::Rem => {
                                        out = Variable::Float(l % r);
                                    }
                                    Operator::CmpGe => {
                                        out = Variable::Bool(l >= r);
                                    }
                                    Operator::CmpNe => {
                                        out = Variable::Bool(l != r);
                                    }
                                    Operator::CmpLe => {
                                        out = Variable::Bool(l <= r);
                                    }
                                    Operator::CmpE => {
                                        out = Variable::Bool(l == r);
                                    }
                                    Operator::CmpG => {
                                        out = Variable::Bool(l > r);
                                    }
                                    Operator::CmpL => {
                                        out = Variable::Bool(l < r);
                                    }
                                    Operator::And => {
                                        out = Variable::Float(f64::from_bits(
                                            l.to_bits() & r.to_bits(),
                                        ));
                                    }
                                    Operator::Or => {
                                        out = Variable::Float(f64::from_bits(
                                            l.to_bits() | r.to_bits(),
                                        ));
                                    }
                                    Operator::Xor => {
                                        out = Variable::Float(f64::from_bits(
                                            l.to_bits() ^ r.to_bits(),
                                        ));
                                    }
                                }
                                _ = 0;
                            }
                            _ => {
                                return Err(format!(
                                    "incompatible types:{:#?} and {:#?}",
                                    left, right
                                )
                                .into());
                            }
                        }
                        _ = 0;
                    }
                    Variable::Pointer(l) => {
                        match rght {
                            Variable::Pointer(r) => match op {
                                Operator::And => {
                                    out = Variable::Bool(l != 0 && r != 0);
                                }
                                Operator::Or => {
                                    out = Variable::Bool(l != 0 || r != 0);
                                }
                                Operator::Xor => {
                                    out = Variable::Bool((l != 0) ^ (r != 0));
                                }
                                Operator::CmpNe => {
                                    out = Variable::Bool(l != r);
                                }
                                Operator::CmpE => {
                                    out = Variable::Bool(l == r);
                                }

                                _ => {
                                    return Err(format!(
                                        "incompatible operation for types:{:#?} and {:#?} {:#?}",
                                        left, right, op
                                    )
                                    .into());
                                }
                            },
                            _ => {
                                return Err(format!(
                                    "incompatible types:{:#?} and {:#?}",
                                    left, right
                                )
                                .into());
                            }
                        }
                        _ = 0;
                    }
                    Variable::String(l) => {
                        match rght {
                            Variable::String(r) => {
                                match op {
                                    Operator::Add => {
                                        let tmp = l.to_string() + &r;
                                        out = Variable::String(tmp.into());
                                    }
                                    Operator::CmpGe => {
                                        out = Variable::Bool(l >= r);
                                    }
                                    Operator::CmpNe => {
                                        out = Variable::Bool(l != r);
                                    }
                                    Operator::CmpLe => {
                                        out = Variable::Bool(l <= r);
                                    }
                                    Operator::CmpE => {
                                        out = Variable::Bool(l == r);
                                    }
                                    Operator::CmpG => {
                                        out = Variable::Bool(l > r);
                                    }
                                    Operator::CmpL => {
                                        out = Variable::Bool(l < r);
                                    }
                                    _ => {
                                        return Err(format!(
                                        "incompatible operation for types:{:#?} and {:#?} {:#?}",
                                        left, right, op
                                    )
                                    .into());
                                    }
                                }
                                _ = 0;
                            }
                            _ => {
                                return Err(format!(
                                    "incompatible types:{:#?} and {:#?}",
                                    left, right
                                )
                                .into());
                            }
                        }
                        _ = 0;
                    }
                }
                output.write(out, &self.data, &mut self.current_frame)?;
            }
            Instruction::Not { input, output } => {
                let inp = input.read(&self.data, &self.current_frame)?;
                let out: Variable;
                match inp {
                    Variable::Bool(x) => {
                        out = Variable::Bool(!x);
                    }
                    Variable::Unsigned(x) => {
                        out = Variable::Unsigned(x.not());
                    }
                    Variable::Signed(x) => {
                        out = Variable::Signed(x.not());
                    }
                    Variable::Float(x) => {
                        out = Variable::Float(f64::from_bits(x.to_bits().not()));
                    }
                    Variable::Pointer(x) => {
                        out = Variable::Bool(!(x != 0));
                    }
                    Variable::String(_x) => {
                        return Err("cannot not a string".into());
                    }
                }
                _ = 0;
                output.write(out, &self.data, &mut self.current_frame)?;
            }
            Instruction::BeginStackFrame { variable_count } => {
                for _ in 0..*variable_count {
                    self.current_frame.push(Variable::Pointer(0));
                }
                _ = 0;
            }
            Instruction::Return { to_return } => {
                let Some((ptr, vals, returned_value)) = self.previous_frames.pop() else {
                    return Err("attempted to return from base of stack".into());
                };
                self.current_frame = vals;
                self.instruction_pointer = ptr;
                if let Some(y) = returned_value.as_ref() {
                    y.write(to_return.clone(), &self.data, &mut self.current_frame)?;
                }
            }
            Instruction::Call {
                to_call,
                arguments,
                output,
            } => {
                let mut new_frame = Vec::new();
                for i in arguments.iter() {
                    new_frame.push(i.read(&self.data, &mut self.current_frame));
                }
                let mut stp = Vec::new();
                std::mem::swap(&mut stp, &mut self.current_frame);
                self.previous_frames
                    .push((self.instruction_pointer, stp, output.clone()));
                self.instruction_pointer = *to_call;
            }
            Instruction::Jmp { to_jump_to } => {
                self.instruction_pointer = *to_jump_to;
            }
            Instruction::JmpIfNot {
                to_jump_to,
                condition,
            } => {
                let cond: bool;
                let v = condition.read(&self.data, &mut self.current_frame)?;
                match v {
                    Variable::Bool(x) => {
                        cond = !x;
                    }
                    Variable::Unsigned(x) => {
                        cond = x != 0;
                    }
                    Variable::Signed(x) => {
                        cond = x != 0;
                    }
                    Variable::Float(x) => {
                        cond = x != 0.0;
                    }
                    Variable::Pointer(x) => {
                        cond = x != 0;
                    }
                    Variable::String(x) => {
                        cond = !x.is_empty();
                    }
                }
                if cond {
                    self.instruction_pointer = *to_jump_to;
                }
            }
            Instruction::Halt => {
                self.halted = true;
            }
            Instruction::Read { to_write_to } => {
                let mut s = String::new();
                std::io::stdin().read_line(&mut s)?;
                to_write_to.write(
                    Variable::String(s.into()),
                    &self.data,
                    &mut self.current_frame,
                )?;
            }
            Instruction::WriteLn { to_write } => {
                let v = to_write.read(&self.data, &self.current_frame)?;
                match v {
                    Variable::Bool(x) => {
                        println!("{x}");
                    }
                    Variable::Unsigned(x) => {
                        println!("{x}");
                    }
                    Variable::Signed(x) => {
                        println!("{x}");
                    }
                    Variable::Float(x) => {
                        println!("{x}");
                    }
                    Variable::Pointer(x) => {
                        println!("{x}");
                    }
                    Variable::String(x) => {
                        println!("{x}");
                    }
                }
            }
            Instruction::Write { to_write } => {
                let v = to_write.read(&self.data, &self.current_frame)?;
                match v {
                    Variable::Bool(x) => {
                        print!("{x}");
                    }
                    Variable::Unsigned(x) => {
                        print!("{x}");
                    }
                    Variable::Signed(x) => {
                        print!("{x}");
                    }
                    Variable::Float(x) => {
                        print!("{x}");
                    }
                    Variable::Pointer(x) => {
                        print!("{x}");
                    }
                    Variable::String(x) => {
                        print!("{x}");
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while !self.halted {
            self.step()?;
        }
        Ok(())
    }

    pub fn new(instructions: Vec<Instruction>, start: usize) -> Self {
        let mut heap = Vec::new();
        for _ in 0..4096 {
            heap.push(Mutex::new(None));
        }
        Self {
            data: ProgramData {
                heap: heap.into(),
                instructions: instructions.into(),
            },
            instruction_pointer: start,
            current_frame: Vec::new(),
            previous_frames: Vec::new(),
            halted: false,
        }
    }
}

impl Operand {
    pub fn read(&self, data: &ProgramData, stack: &[Variable]) -> Result<Variable, Box<dyn Error>> {
        match self {
            Operand::StackVariable { v } => Ok(stack[*v].clone()),
            Operand::FieldAccess { v, of } => {
                if let Some(y) = data.heap[*v].lock().unwrap().as_ref() {
                    Ok(y.fields[*of].clone())
                } else {
                    Err("testing 1 2 3 4".into())
                }
            }
            Operand::ArrayAccess { v, of } => {
                if let Some(y) = data.heap[*v].lock().unwrap().as_ref() {
                    let idx = of.read(data, stack)?;
                    let v = match idx {
                        Variable::Signed(x) => x as usize,
                        Variable::Unsigned(x) => x as usize,
                        _ => {
                            return Err("cannot index by".into());
                        }
                    };
                    Ok(y.fields[v].clone())
                } else {
                    Err("testing 1 2 3 4".into())
                }
            }
            Operand::ConstantUnsigned(v) => Ok(Variable::Unsigned(*v)),
            Operand::ConstantSigned(v) => Ok(Variable::Signed(*v)),
            Operand::ConstantFloat(v) => Ok(Variable::Float(*v)),
            Operand::ConstantString(v) => Ok(Variable::String(v.clone())),
            Operand::NullPtr => Ok(Variable::Pointer(0)),
        }
    }
    pub fn write(
        &self,
        value: Variable,
        data: &ProgramData,
        stack: &mut [Variable],
    ) -> Result<(), Box<dyn Error>> {
        match self {
            Operand::StackVariable { v } => {
                stack[*v] = value;
                Ok(())
            }
            Operand::FieldAccess { v, of } => {
                if let Some(y) = data.heap[*v].lock().unwrap().as_mut() {
                    y.fields[*of] = value;
                    Ok(())
                } else {
                    Err("testing 1 2 3 4".into())
                }
            }
            Operand::ArrayAccess { v, of } => {
                if let Some(y) = data.heap[*v].lock().unwrap().as_mut() {
                    let idx = of.read(data, stack)?;
                    let v = match idx {
                        Variable::Signed(x) => x as usize,
                        Variable::Unsigned(x) => x as usize,
                        _ => {
                            return Err("cannot index by".into());
                        }
                    };
                    y.fields[v] = value;
                    Ok(())
                } else {
                    Err("testing 1 2 3 4".into())
                }
            }
            Operand::ConstantUnsigned(_) => {
                return Err("cannot write to constant".into());
            }
            Operand::ConstantSigned(_) => {
                return Err("cannot write to constant".into());
            }
            Operand::ConstantFloat(_) => {
                return Err("cannot write to constant".into());
            }
            Operand::ConstantString(_) => {
                return Err("cannot write to constant".into());
            }
            Operand::NullPtr => {
                return Err("cannot write to null".into());
            }
        }
    }
}
