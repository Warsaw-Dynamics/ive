//// Shapes on the plane.

import gleam/float
import gleam/io
import gleam/list.{map, type Continue}

pub const pi = 3.14159

/// A shape with a label.
pub type Shape(tag) {
  Circle(radius: Float, tag: tag)
  Rect(width: Float, height: Float, tag: tag)
}

// The area of a shape
pub fn area(shape: Shape(a)) -> Float {
  case shape {
    Circle(radius: r, ..) -> pi *. r *. r
    Rect(width:, height:, ..) -> width *. height
    _ -> 0.0
  }
}

@deprecated("Use total")
pub fn sum(shapes: List(Shape(a)), from start: Float) -> Float {
  let areas = map(shapes, fn(s) { area(s) })
  list.fold(areas, start, float.add)
}

pub fn report(shapes) {
  let pair = #(list.length(shapes), [True])
  sum(shapes, from: 0.0)
  |> float.to_string
  |> io.println
  io.debug(#("count:\t", pair.0))
}
