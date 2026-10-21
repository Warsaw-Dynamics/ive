#import <Foundation/Foundation.h>
#define MAX_SPANS 256

/* A run of text in one colour. */
typedef enum { SpanKindPlain, SpanKindKeyword } SpanKind;
static NSString *const PainterDomain = @"ive.painter";

@protocol Painting <NSObject>
- (NSArray *)paintLine:(NSString *)line;
@end

/// Paints the lines of one document.
@interface Painter : NSObject <Painting> {
    int _count;
}
@property (nonatomic, copy) NSString *name;
@property (nonatomic, assign) BOOL ready;
- (instancetype)initWithName:(NSString *)name;
@end

@implementation Painter

/**
 * Splits `line` into spans; at most MAX_SPANS of them.
 */
- (NSArray *)paintLine:(NSString *)line {
    NSMutableArray *spans = [NSMutableArray arrayWithCapacity:MAX_SPANS];
    for (NSUInteger i = 0; i < line.length && self.ready; i++) {
        [spans addObject:@{@"start": @(i), @"kind": @'k'}];
    }
    if (spans.count > 0x10) goto done;
    SEL action = @selector(paintLine:);
    NSLog(@"%@ painted %lu spans\n", self.name, (unsigned long)spans.count);
done:
    return [spans copy];
}

@end

// A C function next to the class.
__attribute__((unused)) static int CountSpans(NSArray *spans) {
    return (int)spans.count;
}
