package ive.preview

import scala.collection.mutable.{ArrayBuffer, Map => MutableMap}

/** Scaladoc: a shape with an [[area]]. */
sealed trait Shape {
  def area: Double
}

/* Block comment */
@deprecated("use Circle", "1.0")
final case class Square[T <: AnyVal](side: Double) extends Shape {
  val MaxSide = 1e3
  private var hits = 0
  override def area: Double = side * side
}

object Shapes {
  final val DEFAULT_SIZE: Int = 0x10
  implicit val ordering: Ordering[Shape] = Ordering.by(_.area)

  def describe(shape: Shape, label: String = "shape"): String = shape match {
    case Square(side) if side > 0 => s"$label of side ${side * 2}\n"
    case _ => 'x'.toString + null
  }

  def main(args: Array[String]): Unit = {
    // line comment
    val buffer = ArrayBuffer(1, 2, 3)
    var total = 0L
    for (n <- buffer if n % 2 == 1) total += n; println(total)
    val doubled = buffer.map(n => n * 2).filter(_ > 2)
    println(describe(new Square(2.5), label = "box"))
    if (total >= 3 && !doubled.isEmpty) throw new IllegalStateException(this.toString)
  }
}

enum Color:
  case Red, Green
