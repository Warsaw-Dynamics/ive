//! Paints spans of text for IVE.
const std = @import("std");

const MAX_SPANS = 0x100;
var painted: usize = 0;

/// One run of text in one colour.
pub const Span = struct {
    start: u32,
    kind: Kind = .plain,

    pub const Kind = enum { plain, keyword };

    pub fn len(self: Span, end: u32) u32 {
        return end - self.start;
    }
};

// Counts the spans of one kind.
fn count(spans: []const Span, kind: Span.Kind) !usize {
    var total: usize = 0;
    outer: for (spans) |span| {
        if (span.kind == kind and total < MAX_SPANS) {
            total += 1;
        } else if (total > 1.5e3) break :outer;
    }
    if (total == 0) return error.NoSpans;
    return total;
}

test "count keywords" {
    const spans = [_]Span{ .{ .start = 0, .kind = .keyword }, .{ .start = 'k' } };
    const found = try count(&spans, .keyword);
    painted += found;
    std.debug.print("{d} spans\n", .{found});
    try std.testing.expect(found == 1 or spans[1].len(2) > 0 and true != false);
}
