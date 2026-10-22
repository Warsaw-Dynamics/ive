use super::keys_at;

#[test]
fn should_tell_objc_method_declaration_from_message_send_and_parameter() {
    // given
    let source = "@implementation A\n- (void)setName:(NSString *)name { [self.store addObject:name]; }\n@end";

    // when
    let keys = keys_at("Objective-C", source, ["setName", "name {", "addObject", "store"]);

    // then
    assert_eq!(
        keys,
        [Some("OBJC.METHOD_DECLARATION"), Some("OBJC.PARAMETER"), Some("OBJC.MESSAGE_SEND"), Some("OBJC.PROPERTY")]
    );
}

#[test]
fn should_mark_objc_at_keyword_class_protocol_and_property() {
    // given
    let source = "@interface Painter : NSObject <Painting>\n@property (nonatomic) BOOL ready;\n@end";

    // when
    let keys = keys_at("Objective-C", source, ["@interface", "Painter", "Painting", "ready", "BOOL"]);

    // then
    assert_eq!(
        keys,
        [
            Some("OBJC.AT_KEYWORD"),
            Some("OBJC.CLASS"),
            Some("OBJC.PROTOCOL"),
            Some("OBJC.PROPERTY"),
            Some("OBJC.PRIMITIVE_TYPE"),
        ]
    );
}

#[test]
fn should_mark_objc_selector_string_and_self() {
    // given
    let source = "void f(void) { SEL s = @selector(paint:); NSString *t = @\"x\"; [self run]; }";

    // when
    let keys = keys_at("Objective-C", source, ["paint:", "@\"x\"", "self", "run"]);

    // then
    assert_eq!(keys, [Some("OBJC.SELECTOR"), Some("OBJC.STRING"), Some("OBJC.KEYWORD"), Some("OBJC.MESSAGE_SEND")]);
}
