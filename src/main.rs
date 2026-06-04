use crate::interpreter::{Instruction, Machine};
use crate::interpreter::{Operand, Operator};
pub mod interpreter;

fn main() {
    let list = vec![
        Instruction::BeginStackFrame { variable_count: 16 },
        Instruction::Move {
            to: Operand::StackVariable { v: 0 },
            from: Operand::ConstantSigned(10),
        },
        Instruction::Move {
            to: Operand::StackVariable { v: 1 },
            from: Operand::ConstantSigned(15),
        },
        Instruction::BinaryOperator {
            op: Operator::Add,
            left: Operand::StackVariable { v: 0 },
            right: Operand::StackVariable { v: 1 },
            output: Operand::StackVariable { v: 2 },
        },
        Instruction::WriteLn {
            to_write: Operand::StackVariable { v: 2 },
        },
        Instruction::Move {
            to: Operand::StackVariable { v: 3 },
            from: Operand::ConstantString("hi toast,".into()),
        },
        Instruction::Move {
            to: Operand::StackVariable { v: 4 },
            from: Operand::ConstantString(" i love you <3".into()),
        },
        Instruction::BinaryOperator {
            op: Operator::Add,
            left: Operand::StackVariable { v: 3 },
            right: Operand::StackVariable { v: 4 },
            output: Operand::StackVariable { v: 5 },
        },
        Instruction::WriteLn {
            to_write: Operand::StackVariable { v: 5 },
        },
        Instruction::Halt,
    ];
    let mut prg = Machine::new(list.into(), 0);
    prg.run().unwrap();
}
