import 'dart:math' as math;

const int maxSize = 0x100;
const SCALE = 2;

/// A shape with an [area].
abstract interface class Shape {
  double get area;
}

/* Block comment */
@immutable
class Circle<T extends num> implements Shape {
  static const defaultRadius = 1.5;
  final double radius;

  const Circle(this.radius);

  @override
  double get area => math.pi * radius * radius;
}

enum Color { red, green }

// Line comment
Future<int> describe(List<Shape> shapes, {required String label}) async {
  var total = 0.0;
  outer:
  for (final shape in shapes) {
    if (shape is! Circle) continue outer;
    total += shape.area;
  }
  final names = <String>['a', 'b']..add(label);
  names.add("$label: ${total.toStringAsFixed(2)}\n");
  print(names[0]);
  print(#ready);
  if (total > Circle.defaultRadius && shapes.isNotEmpty) {
    return Color.red.index * SCALE;
  }
  return null ?? Circle(1.0).radius.round();
}
