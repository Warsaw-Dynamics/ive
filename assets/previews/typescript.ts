interface Shape {
  area(): number;
}

type Point = { x: number; y: number };

enum Color { Red, Green }

@sealed
class Circle<T extends Point> implements Shape {
  private readonly radius: number;

  constructor(radius: number, public center?: T) {
    this.radius = radius;
  }

  area(): number {
    return Math.PI * this.radius ** 2; // line comment
  }
}

const shapes: Array<Shape> = [new Circle(2)];
