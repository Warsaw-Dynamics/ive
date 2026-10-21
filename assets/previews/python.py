@decorator(param=1)
def f(x):
    """
    Syntax Highlighting Demo
    @param x Parameter
    """

    def nested_func(y):
        print(y + 1)

    s = ("Test", 2+3, {'a': 'b'}, f'{x!s:{"^10"}}', f"{1:01d}")   # Comment
    f(s[0].lower())
    nested_func(42)

class Foo:
    tags: List[str]

    def __init__(self: Foo):
        byte_string: bytes = b'newline:\n also newline:\x0a'
        text_string = u"Cyrillic Я is \u042f. Oops: \u042g"
        self.make_sense(whatever=1)

    def make_sense[T](self, whatever: T):
        self.sense = whatever

x = len('abc')
type my_int = int
print(f.__doc__)
