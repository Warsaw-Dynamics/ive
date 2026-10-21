//! Module doc comment
use std::collections::HashMap;

/// A point in 2D space.
#[derive(Debug, Clone, Copy)]
pub struct Point<'a, T> {
    pub x: T,
    label: &'a str,
}

pub trait Shape {
    fn area(&self) -> f64;
}

const MAX_POINTS: usize = 1_000;
static GREETING: &str = "Hello\tworld\n";

enum Direction { North, South }

fn distance<T: Into<f64> + Copy>(from: Point<T>, to: Point<T>) -> f64 {
    let dx = from.x.into() - to.x.into();
    /* block comment */
    let mut seen = HashMap::new();
    seen.insert('c', Direction::North);
    println!("{dx} {}", MAX_POINTS);
    dx.abs() // line comment
}
