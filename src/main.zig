const std = @import("std");
const Io = std.Io;

const _86dos = @import("_86dos");
pub const interp = @import("interpreter.zig");
pub const ops = interp.Opb;
pub const isb = interp.Insb;
pub fn main(init: std.process.Init) !void {
    const arena: std.mem.Allocator = init.arena.allocator();
    var instructions: std.array_list.Aligned(interp.Instruction, null) = try std.array_list.Aligned(interp.Instruction, null).initCapacity(arena, 16);
    try instructions.append(arena, isb.in_begin_stack_frame(16));
    try instructions.append(arena, isb.in_move(ops.new_stack(0), ops.new_signed(10)));
    try instructions.append(arena, isb.in_move(ops.new_stack(1), ops.new_signed(15)));
    try instructions.append(arena, isb.in_binary_operator(interp.Operator.Add, ops.new_stack(0), ops.new_stack(1), ops.new_stack(2)));
    try instructions.append(arena, isb.in_print_integer(ops.new_stack(2), true));
    try instructions.append(arena, .Halt);
    var machine = try interp.create_machine(arena, 0, instructions.items);
    try interp.machine_run(arena, &machine);
}

test "simple test" {
    const gpa = std.testing.allocator;
    var list: std.ArrayList(i32) = .empty;
    defer list.deinit(gpa); // Try commenting this out and see if zig detects the memory leak!
    try list.append(gpa, 42);
    try std.testing.expectEqual(@as(i32, 42), list.pop());
}

test "fuzz example" {
    try std.testing.fuzz({}, testOne, .{});
}

fn testOne(context: void, smith: *std.testing.Smith) !void {
    _ = context;
    // Try passing `--fuzz` to `zig build test` and see if it manages to fail this test case!

    const gpa = std.testing.allocator;
    var list: std.ArrayList(u8) = .empty;
    defer list.deinit(gpa);
    while (!smith.eos()) switch (smith.value(enum { add_data, dup_data })) {
        .add_data => {
            const slice = try list.addManyAsSlice(gpa, smith.value(u4));
            smith.bytes(slice);
        },
        .dup_data => {
            if (list.items.len == 0) continue;
            if (list.items.len > std.math.maxInt(u32)) return error.SkipZigTest;
            const len = smith.valueRangeAtMost(u32, 1, @min(32, list.items.len));
            const off = smith.valueRangeAtMost(u32, 0, @intCast(list.items.len - len));
            try list.appendSlice(gpa, list.items[off..][0..len]);
            try std.testing.expectEqualSlices(
                u8,
                list.items[off..][0..len],
                list.items[list.items.len - len ..],
            );
        },
    };
}
