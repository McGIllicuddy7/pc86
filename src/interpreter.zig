const std = @import("std");
pub const Operand = union(enum) { Bool: bool, Signed: i64, Unsigned: u64, Float: f64, String: []u8, StackVariable: usize, FieldAccess: struct { v: usize, offset: usize }, ArrayAccessConstOffset: struct {
    v: usize,
    offset: usize,
}, ArrayAccess: struct {
    v: usize,
    offset: usize,
} };
pub const ReadError = error{ IsBool, IsSigned, IsUnsigned, IsFloat, IsString, IsPtr, IsNotWritable, IsNull, InvalidOperation, StackUnderFlow };
pub const Variable = union(enum) {
    Bool: bool,
    Signed: i64,
    Unsigned: u64,
    Float: f64,
    String: []u8,
    HeapPtr: usize,
    pub fn read_as_unsigned(self: *const Variable) !u64 {
        switch (self.*) {
            .Bool => {
                return ReadError.IsBool;
            },
            .Signed => {
                return ReadError.IsSigned;
            },
            .Unsigned => |*s| {
                return s.*;
            },
            .Float => {
                return ReadError.IsFloat;
            },
            .String => {
                return ReadError.IsString;
            },
            .HeapPtr => {
                return ReadError.IsPtr;
            },
        }
    }
    pub fn read_as_signed(self: *const Variable) !i64 {
        switch (self.*) {
            .Bool => {
                return ReadError.IsBool;
            },
            .Signed => |*ins_data| {
                return ins_data.*;
            },
            .Unsigned => {
                return ReadError.IsUnsigned;
            },
            .Float => {
                return ReadError.IsFloat;
            },
            .String => {
                return ReadError.IsString;
            },
            .HeapPtr => {
                return ReadError.IsPtr;
            },
        }
    }
    pub fn read_as_float(self: *const Variable) !f64 {
        switch (self.*) {
            .Bool => {
                return ReadError.IsBool;
            },
            .Signed => {
                return ReadError.IsSigned;
            },
            .Unsigned => {
                return ReadError.IsUnsigned;
            },
            .Float => |*ins_data| {
                return ins_data.*;
            },
            .String => {
                return ReadError.IsString;
            },
            .HeapPtr => {
                return ReadError.IsPtr;
            },
        }
    }
    pub fn read_as_bool(self: *const Variable) !bool {
        switch (self.*) {
            .Bool => |*ins_data| {
                return ins_data.*;
            },
            .Signed => {
                return ReadError.IsSigned;
            },
            .Unsigned => {
                return ReadError.IsUnsigned;
            },
            .Float => {
                return ReadError.IsBool;
            },
            .String => {
                return ReadError.IsString;
            },
            .HeapPtr => {
                return ReadError.IsPtr;
            },
        }
    }
    pub fn read_as_string(self: *const Variable) ![]u8 {
        switch (self.*) {
            .Bool => {
                return ReadError.IsBool;
            },
            .Signed => {
                return ReadError.IsSigned;
            },
            .Unsigned => {
                return ReadError.IsUnsigned;
            },
            .Float => {
                return ReadError.IsFloat;
            },
            .String => |*ins_data| {
                return ins_data.*;
            },
            .HeapPtr => {
                return ReadError.IsPtr;
            },
        }
    }
    pub fn read_as_ptr(self: *const Variable) !usize {
        switch (self.*) {
            .Bool => {
                return ReadError.IsBool;
            },
            .Signed => {
                return ReadError.IsSigned;
            },
            .Unsigned => {
                return ReadError.IsUnsigned;
            },
            .Float => {
                return ReadError.IsFloat;
            },
            .String => {
                return ReadError.IsString;
            },
            .HeapPtr => |*ins_data| {
                return ins_data.*;
            },
        }
    }

    pub fn write_var(self: *Variable, v: Variable) !void {
        self.* = v;
    }
};

pub const HeapValue = struct {
    is_valid: bool,
    gc_reachable: bool,
    type_info: *const Type,
    fields: []Variable,
};

pub const Type = union(enum) { Bool, Signed, Unsigned, Float, String, Struct: struct { fields: []struct {
    type_info: *const Type,
    name: []u8,
} } };
pub const Heap = []HeapValue;
pub const Stack = struct {
    variables: [16384]Variable,
    base_ptr: usize,
    top_ptr: usize,
};
pub const StackData = struct {
    ip: usize,
    top_ptr: usize,
    base_ptr: usize,
    returned_value: ?Operand,
};
pub const Machine = struct {
    stack: Stack,
    heap: Heap,
    instructions: []Instruction,
    instruction_ptr: usize,
    call_stack: std.array_list.Aligned(StackData, null),
    halted: bool,
};

pub fn read_operand(self: *const Operand, machine: *Machine) !Variable {
    switch (self.*) {
        .Bool => |*ins_data| {
            return Variable{ .Bool = ins_data.* };
        },
        .Signed => |*ins_data| {
            return Variable{ .Signed = ins_data.* };
        },
        .Unsigned => |*ins_data| {
            return Variable{ .Unsigned = ins_data.* };
        },
        .Float => |*ins_data| {
            return Variable{ .Float = ins_data.* };
        },
        .String => |*ins_data| {
            return Variable{ .String = ins_data.* };
        },
        .ArrayAccess => |*ins_data| {
            const ptr = try machine.stack.variables[machine.stack.base_ptr + ins_data.v].read_as_ptr();
            const offset = try machine.stack.variables[machine.stack.base_ptr + ins_data.offset].read_as_unsigned();
            if (ptr == 0) {
                return ReadError.IsNull;
            }
            return machine.heap[ptr].fields[offset];
        },
        .ArrayAccessConstOffset => |*ins_data| {
            const ptr = try machine.stack.variables[machine.stack.base_ptr + ins_data.v].read_as_ptr();
            const offset = ins_data.offset;
            if (ptr == 0) {
                return ReadError.IsNull;
            }
            return machine.heap[ptr].fields[offset];
        },
        .FieldAccess => |*ins_data| {
            const ptr = try machine.stack.variables[machine.stack.base_ptr + ins_data.v].read_as_ptr();
            const offset = ins_data.offset;
            if (ptr == 0) {
                return ReadError.IsNull;
            }
            return machine.heap[ptr].fields[offset];
        },
        .StackVariable => |*ins_data| {
            return machine.stack.variables[machine.stack.base_ptr + ins_data.*];
        },
    }
}
pub fn write_operand(self: *const Operand, machine: *Machine, vs: Variable) !void {
    switch (self.*) {
        .Bool => {
            return ReadError.IsNotWritable;
        },
        .Signed => {
            return ReadError.IsNotWritable;
        },
        .Unsigned => {
            return ReadError.IsNotWritable;
        },
        .Float => {
            return ReadError.IsNotWritable;
        },
        .String => {
            return ReadError.IsNotWritable;
        },
        .ArrayAccess => |*ins_data| {
            const ptr = try machine.stack.variables[machine.stack.base_ptr + ins_data.v].read_as_ptr();
            const offset = try machine.stack.variables[machine.stack.base_ptr + ins_data.offset].read_as_unsigned();
            if (ptr == 0) {
                return ReadError.IsNull;
            }
            try machine.heap[ptr].fields[offset].write_var(vs);
        },
        .ArrayAccessConstOffset => |*ins_data| {
            const ptr = try machine.stack.variables[machine.stack.base_ptr + ins_data.v].read_as_ptr();
            const offset = ins_data.offset;
            if (ptr == 0) {
                return ReadError.IsNull;
            }
            try machine.heap[ptr].fields[offset].write_var(vs);
        },
        .FieldAccess => |*ins_data| {
            const ptr = try machine.stack.variables[machine.stack.base_ptr + ins_data.v].read_as_ptr();
            const offset = ins_data.offset;
            if (ptr == 0) {
                return ReadError.IsNull;
            }
            try machine.heap[ptr].fields[offset].write_var(vs);
        },
        .StackVariable => |*ins_data| {
            try machine.stack.variables[machine.stack.base_ptr + ins_data.*].write_var(vs);
        },
    }
}

pub const Operator = enum {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    LessOrEq,
    GreaterOrEq,
    Equal,
    NotEqual,
    Less,
    Greater,
    And,
    Or,
    Xor,
};

pub const Instruction = union(enum) {
    Nop,
    Move: struct {
        to: Operand,
        from: Operand,
    },
    BinaryOperator: struct {
        op: Operator,
        left: Operand,
        right: Operand,
        output: Operand,
    },
    Call: struct {
        to_call: usize,
        arguments: []Operand,
        to_return: ?Operand,
    },
    Return: struct { to_return: Operand },
    JmpIfNot: struct {
        target: usize,
        if_not: Operand,
    },
    Jmp: struct {
        target: usize,
    },
    Not: struct {
        input: Operand,
        output: Operand,
    },
    BeginStackFrame: struct {
        variable_count: usize,
    },
    PrintInteger: struct {
        to_print: Operand,
        new_line: bool,
    },
    PrintUnsignedInteger: struct {
        to_print: Operand,
        new_line: bool,
    },
    PrintFloat: struct { to_print: Operand, new_line: bool },
    PrintString: struct {
        to_print: Operand,
        new_line: bool,
    },
    Halt,
};
pub fn machine_step(allocator: std.mem.Allocator, machine: *Machine) !void {
    const ins = machine.instructions[machine.instruction_ptr];
    machine.instruction_ptr += 1;
    switch (ins) {
        .BeginStackFrame => |*ins_data| {
            machine.stack.top_ptr += ins_data.variable_count;
        },
        .BinaryOperator => |*ins_data| {
            const v0 = try read_operand(&ins_data.left, machine);
            switch (v0) {
                .Bool => |l| {
                    const r = try (try read_operand(&ins_data.right, machine)).read_as_bool();
                    switch (ins_data.op) {
                        .Add, .Sub, .Mul, .Div, .Rem, .LessOrEq, .GreaterOrEq, .Less, .Greater => {
                            return ReadError.InvalidOperation;
                        },
                        .Equal => {
                            const tmp = l == r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .NotEqual => {
                            const tmp = l != r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .And => {
                            const tmp = l and r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Or => {
                            const tmp = l or r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Xor => {
                            const tmp: bool = l ^ r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                    }
                },
                .Signed => |l| {
                    const r = try (try read_operand(&ins_data.right, machine)).read_as_signed();
                    switch (ins_data.op) {
                        .Add => {
                            const tmp: i64 = l + r;
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                        .Sub => {
                            const tmp: i64 = l - r;
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                        .Div => {
                            const tmp: i64 = @divFloor(l, r);
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                        .Mul => {
                            const tmp: i64 = l * r;
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                        .Rem => {
                            const tmp: i64 = @rem(l, r);
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                        .LessOrEq => {
                            const tmp: bool = l <= r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .GreaterOrEq => {
                            const tmp: bool = l >= r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Equal => {
                            const tmp: bool = l == r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .NotEqual => {
                            const tmp: bool = l != r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Less => {
                            const tmp: bool = l < r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Greater => {
                            const tmp: bool = l > r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .And => {
                            const tmp: i64 = l & r;
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                        .Or => {
                            const tmp: i64 = l | r;
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                        .Xor => {
                            const tmp: i64 = l ^ r;
                            try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                        },
                    }
                },
                .Unsigned => |l| {
                    const r = try (try read_operand(&ins_data.right, machine)).read_as_unsigned();
                    switch (ins_data.op) {
                        .Add => {
                            const tmp: u64 = l + r;
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                        .Sub => {
                            const tmp: u64 = l - r;
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                        .Div => {
                            const tmp: u64 = @divFloor(l, r);
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                        .Mul => {
                            const tmp: u64 = l * r;
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                        .Rem => {
                            const tmp: u64 = @rem(l, r);
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                        .LessOrEq => {
                            const tmp: bool = l <= r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .GreaterOrEq => {
                            const tmp: bool = l >= r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Equal => {
                            const tmp: bool = l == r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .NotEqual => {
                            const tmp: bool = l != r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Less => {
                            const tmp: bool = l < r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Greater => {
                            const tmp: bool = l > r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .And => {
                            const tmp: u64 = l & r;
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                        .Or => {
                            const tmp: u64 = l | r;
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                        .Xor => {
                            const tmp: u64 = l ^ r;
                            try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                        },
                    }
                },
                .Float => |l| {
                    const r = try (try read_operand(&ins_data.right, machine)).read_as_float();
                    switch (ins_data.op) {
                        .Add => {
                            const tmp: f64 = l + r;
                            try write_operand(&ins_data.output, machine, Variable{ .Float = tmp });
                        },
                        .Sub => {
                            const tmp: f64 = l - r;
                            try write_operand(&ins_data.output, machine, Variable{ .Float = tmp });
                        },
                        .Div => {
                            const tmp: f64 = l / r;
                            try write_operand(&ins_data.output, machine, Variable{ .Float = tmp });
                        },
                        .Mul => {
                            const tmp: f64 = l * r;
                            try write_operand(&ins_data.output, machine, Variable{ .Float = tmp });
                        },
                        .Rem => {
                            return ReadError.InvalidOperation;
                        },
                        .LessOrEq => {
                            const tmp: bool = l <= r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .GreaterOrEq => {
                            const tmp: bool = l >= r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Equal => {
                            const tmp: bool = l == r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .NotEqual => {
                            const tmp: bool = l != r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Less => {
                            const tmp: bool = l < r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Greater => {
                            const tmp: bool = l > r;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .And => {
                            return ReadError.InvalidOperation;
                        },
                        .Or => {
                            return ReadError.InvalidOperation;
                        },
                        .Xor => {
                            return ReadError.InvalidOperation;
                        },
                    }
                },
                .String => |l| {
                    const r = try (try read_operand(&ins_data.right, machine)).read_as_string();
                    _ = r;
                    _ = l;
                    switch (ins_data.op) {
                        .Sub, .Mul, .Div, .Rem, .And, .Or, .Xor => {
                            return ReadError.InvalidOperation;
                        },
                        .Add => {
                            return ReadError.InvalidOperation;
                        },
                        .LessOrEq => {
                            return ReadError.InvalidOperation;
                        },
                        .GreaterOrEq => {
                            return ReadError.InvalidOperation;
                        },
                        .Equal => {
                            return ReadError.InvalidOperation;
                        },
                        .NotEqual => {
                            return ReadError.InvalidOperation;
                        },
                        .Less => {
                            return ReadError.InvalidOperation;
                        },
                        .Greater => {
                            return ReadError.InvalidOperation;
                        },
                    }
                },
                .HeapPtr => |l| {
                    const r = try (try read_operand(&ins_data.right, machine)).read_as_ptr();
                    switch (ins_data.op) {
                        .Equal => {
                            const tmp = r == l;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .NotEqual => {
                            const tmp = r != l;
                            try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                        },
                        .Add, .Sub, .Mul, .Div, .Rem, .LessOrEq, .GreaterOrEq, .Less, .Greater, .And, .Or, .Xor => {
                            return ReadError.InvalidOperation;
                        },
                    }
                },
            }
        },
        .Call => |*ins_data| {
            var v = StackData{ .ip = machine.instruction_ptr, .base_ptr = machine.stack.base_ptr, .top_ptr = machine.stack.top_ptr, .returned_value = null };
            machine.stack.base_ptr = machine.stack.top_ptr;
            machine.stack.top_ptr += ins_data.arguments.len;
            var count: usize = 0;
            for (ins_data.arguments) |i| {
                const tmp = try read_operand(&i, machine);
                machine.stack.variables[machine.stack.base_ptr + count] = tmp;
                count += 1;
            }
            v.returned_value = ins_data.to_return;
            try machine.call_stack.append(allocator, v);
            machine.instruction_ptr = ins_data.to_call;
            machine.instruction_ptr = ins_data.to_call;
        },
        .Return => |*ins_data| {
            var v0: Variable = undefined;
            if (read_operand(&ins_data.to_return, machine)) |x| {
                v0 = x;
            } else |err| {
                return err;
                //v0 = Variable{ .HeapPtr = 0 };
            }
            const stck = machine.call_stack.pop() orelse {
                return ReadError.StackUnderFlow;
            };
            if (stck.returned_value != null) {
                const output: Operand = stck.returned_value.?;
                try write_operand(&output, machine, v0);
            }
            machine.instruction_ptr = stck.ip;
            machine.stack.top_ptr = stck.top_ptr;
            machine.stack.base_ptr = stck.base_ptr;
        },
        .Jmp => |*ins_data| {
            machine.instruction_ptr = ins_data.target;
        },
        .JmpIfNot => |*ins_data| {
            const v = try read_operand(&ins_data.if_not, machine);
            switch (v) {
                .Bool => |*t| {
                    if (!t.*) {
                        machine.instruction_ptr = ins_data.target;
                    }
                },
                .Float => |*t| {
                    if (t.* == 0.0) {
                        machine.instruction_ptr = ins_data.target;
                    }
                },
                .HeapPtr => |*t| {
                    if (t.* == 0) {
                        machine.instruction_ptr = ins_data.target;
                    }
                },
                .Signed => |*t| {
                    if (t.* == 0) {
                        machine.instruction_ptr = ins_data.target;
                    }
                },
                .String => |*t| {
                    if (t.len == 0) {
                        machine.instruction_ptr = ins_data.target;
                    }
                },
                .Unsigned => |*t| {
                    if (t.* == 0) {
                        machine.instruction_ptr = ins_data.target;
                    }
                },
            }
        },
        .Not => |*ins_data| {
            const v0 = try read_operand(&ins_data.input, machine);
            switch (v0) {
                .Bool => |*y| {
                    const tmp = !y.*;
                    try write_operand(&ins_data.output, machine, Variable{ .Bool = tmp });
                },
                .Float => {
                    return ReadError.InvalidOperation;
                    // const tmp = ~y.*;
                    //try write_operand(ins_data.output, machine, Variable{ .Signed = tmp });*/
                },
                .Signed => |*y| {
                    const tmp = ~y.*;
                    try write_operand(&ins_data.output, machine, Variable{ .Signed = tmp });
                },
                .String, .HeapPtr => {
                    return ReadError.InvalidOperation;
                },
                .Unsigned => |*y| {
                    const tmp = ~y.*;
                    try write_operand(&ins_data.output, machine, Variable{ .Unsigned = tmp });
                },
            }
        },
        .PrintInteger => |*ins_data| {
            const tmp = try read_operand(&ins_data.to_print, machine);
            const t2 = try tmp.read_as_signed();
            if (ins_data.new_line) {
                std.debug.print("{}\n", .{t2});
            } else {
                std.debug.print("{}", .{t2});
            }
        },
        .PrintUnsignedInteger => |*ins_data| {
            const tmp = try read_operand(&ins_data.to_print, machine);
            const t2 = try tmp.read_as_unsigned();
            if (ins_data.new_line) {
                std.debug.print("{}\n", .{t2});
            } else {
                std.debug.print("{}", .{t2});
            }
        },
        .PrintFloat => |*ins_data| {
            const tmp = try read_operand(&ins_data.to_print, machine);
            const t2 = try tmp.read_as_float();
            if (ins_data.new_line) {
                std.debug.print("{}\n", .{t2});
            } else {
                std.debug.print("{}", .{t2});
            }
        },
        .PrintString => |*ins_data| {
            const tmp = try read_operand(&ins_data.to_print, machine);
            const t2 = try tmp.read_as_string();
            if (ins_data.new_line) {
                std.debug.print("{s}\n", .{t2});
            } else {
                std.debug.print("{s}", .{t2});
            }
        },
        .Move => |*ins_data| {
            const tmp = try read_operand(&ins_data.from, machine);
            try write_operand(&ins_data.to, machine, tmp);
        },
        .Nop => {},
        .Halt => {
            machine.halted = true;
        },
    }
}

pub fn machine_run(allocator: std.mem.Allocator, machine: *Machine) !void {
    while (!machine.halted) {
        try machine_step(allocator, machine);
    }
}

pub fn create_machine(allocator: std.mem.Allocator, start: usize, instructions: []Instruction) !Machine {
    return Machine{ .instructions = instructions, .instruction_ptr = start, .heap = try allocator.alloc(HeapValue, 4096), .halted = false, .call_stack = try std.array_list.Aligned(StackData, null).initCapacity(allocator, 16), .stack = undefined };
}

pub const Insb = struct {
    pub fn in_nop() Instruction {
        return .Nop;
    }

    pub fn in_move(to: Operand, from: Operand) Instruction {
        return .{ .Move = .{ .from = from, .to = to } };
    }

    pub fn in_binary_operator(op: Operator, left: Operand, right: Operand, output: Operand) Instruction {
        return .{ .BinaryOperator = .{ .op = op, .left = left, .output = output, .right = right } };
    }

    pub fn in_call(to_call: usize, arguments: []Operand, to_return: ?Operand) Instruction {
        return .{ .Call = .{ .arguments = arguments, .to_call = to_call, .to_return = to_return } };
    }

    pub fn in_return(to_return: ?Operand) Instruction {
        const v = to_return orelse
            Operand{ .Unsigned = 0 };
        return Instruction{ .Return = .{ .to_return = v } };
    }

    pub fn in_jmp_if_not(target: usize, if_not: Operand) Instruction {
        return Instruction{ .JmpIfNot = .{ .if_not = if_not, .target = target } };
    }

    pub fn in_jmp(target: usize) Instruction {
        return Instruction{ .Jmp = .{ .target = target } };
    }

    pub fn in_not(input: Operand, output: Operand) Instruction {
        return Instruction{ .Not = .{ .input = input, .output = output } };
    }

    pub fn in_begin_stack_frame(variable_count: usize) Instruction {
        return Instruction{ .BeginStackFrame = .{ .variable_count = variable_count } };
    }

    pub fn in_print_integer(to_print: Operand, new_line: bool) Instruction {
        return Instruction{ .PrintInteger = .{ .new_line = new_line, .to_print = to_print } };
    }

    pub fn in_print_unsigned(to_print: Operand, new_line: bool) Instruction {
        return Instruction{ .PrintUnsignedInteger = .{ .new_line = new_line, .to_print = to_print } };
    }

    pub fn in_print_float(to_print: Operand, new_line: bool) Instruction {
        return Instruction{ .PrintFloat = .{ .new_line = new_line, .to_print = to_print } };
    }

    pub fn in_print_string(to_print: Operand, new_line: bool) Instruction {
        return Instruction{ .PrintString = .{ .new_line = new_line, .to_print = to_print } };
    }

    pub fn in_halt() Instruction {
        return Instruction{.Halt};
    }
};

pub const Opb = struct {
    pub fn new_bool(b: bool) Operand {
        return Operand{ .Bool = b };
    }

    pub fn new_signed(b: i64) Operand {
        return Operand{ .Signed = b };
    }

    pub fn new_unsigned(b: u64) Operand {
        return Operand{ .Unsigned = b };
    }

    pub fn new_float(b: f64) Operand {
        return Operand{ .Float = b };
    }

    pub fn new_stack(v: usize) Operand {
        return Operand{ .StackVariable = v };
    }

    pub fn new_string(st: []u8) Operand {
        return Operand{ .String = st };
    }

    pub fn new_field_access(v: usize, offset: usize) Operand {
        return Operand{ .FieldAccess = .{ .v = v, .offset = offset } };
    }

    pub fn new_array_access(v: usize, offset: usize) Operand {
        return Operand{ .ArrayAccess = .{ .offset = offset, .v = v } };
    }

    pub fn new_array_access_constant(v: usize, offset: usize) Operand {
        return Operand{ .ArrayAccessConstOffset = .{ .offset = offset, .v = v } };
    }
};
