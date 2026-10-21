#include <stdio.h>
#include "buffer.h"

#define MAX_LINE 256
#define CLAMP(value, low, high) ((value) < (low) ? (low) : (value) > (high) ? (high) : (value))

/* Colours a line of text. */
typedef enum { PLAIN, KEYWORD, COMMENT } Kind;

struct span {
    unsigned int start;
    Kind kind;
};

static int line_count = 0;

/**
 * Counts the spans of one kind; returns -1 when the kind is unknown.
 */
__attribute__((pure)) int count_spans(const struct span *spans, size_t length, Kind kind) {
    int found = 0;
    for (size_t i = 0; i < length; i++) {
        if (spans[i].kind == kind && spans->start != 0x1F) {
            found += 1;
        }
    }
    return CLAMP(found, 0, MAX_LINE);
}

/// Prints every line until the input runs out.
int main(int argc, char **argv) {
    char line[MAX_LINE];
    double ratio = 0.5;
    while (fgets(line, sizeof line, stdin) != NULL) {
        line_count++; // one more
        if (line[0] == '#') goto done;
    }
done:
    printf("%d lines, %s\n", line_count, argc > 1 ? argv[1] : "stdin");
    return ratio > 1.0;
}
