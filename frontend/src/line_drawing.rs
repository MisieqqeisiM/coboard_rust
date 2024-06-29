use itertools::Itertools;
use leptos::html::li;
use std::f64::consts::{FRAC_PI_2, TAU};

use nalgebra::{ComplexField, Matrix2, Point2, Rotation, Rotation2, Vector2};

type Point = Point2<f64>;
type Vector = Vector2<f64>;

pub fn line_into_triangles(line: Vec<Point>, width: f64) -> Vec<Point> {
    if let [a] = line[..] {
        circle(a, width)
    } else {
        line.iter()
            .tuple_windows()
            .flat_map(|(a, b)| rectangle(a.to_owned(), b.to_owned(), width))
            .chain(
                line.iter()
                    .tuple_windows()
                    .flat_map(|(a, b, c)| elbow(a.to_owned(), b.to_owned(), c.to_owned(), width)),
            )
            .chain(cap(line[0], line[1], width))
            .chain(cap(line[line.len() - 1], line[line.len() - 2], width))
            .collect()
    }
}

fn rectangle(from: Point, to: Point, width: f64) -> Vec<Point> {
    let dir = (to - from).normalize();
    let perp = Rotation2::new(FRAC_PI_2) * dir;

    let p1 = from + perp * width;
    let p2 = from - perp * width;
    let p3 = to + perp * width;
    let p4 = to - perp * width;

    vec![p1, p2, p3, p3, p2, p4]
}

const ANGLE_RES: f64 = 0.3;
const MIN_ANGLE: f64 = 0.001;

fn cap(from: Point, to: Point, width: f64) -> Vec<Point> {
    let dir = (to - from).normalize();
    let perp = Rotation2::new(FRAC_PI_2) * dir;

    let p1 = from + perp * width;
    let p2 = from - perp * width;

    arc(p1, from, p2)
}

fn circle(a: Point, width: f64) -> Vec<Point> {
    let segment_count: u32 = (TAU / ANGLE_RES).ceil() as u32;
    let start = a + Vector2::new(width, 0.0);
    let mut points = vec![start];
    let rotation = Rotation2::new(TAU / segment_count as f64);
    for _ in 0..segment_count {
        let prev = points.last().unwrap();
        points.push(a + rotation * (prev - a));
    }
    points.push(start);
    points
        .into_iter()
        .tuple_windows()
        .flat_map(|(x, y)| [x, y, a])
        .collect()
}


/// Constructs arc centered in `b` ranging from point `a` to `c` in counterclockwise direction
fn arc(a: Point, b: Point, c: Point) -> Vec<Point> {
    let angle = ccw_angle(&(a - b), &(c - b));
    if angle < MIN_ANGLE {
        return vec![];
    }
    let segment_count = (angle / ANGLE_RES).ceil() as u32;
    let rotation = Rotation2::new(angle / segment_count as f64);
    let mut points = vec![a];
    for _ in 0..(segment_count - 1) {
        let prev = points.last().unwrap();
        points.push(b + rotation * (prev - b));
    }
    points.push(c);
    points
        .into_iter()
        .tuple_windows()
        .flat_map(|(x, y)| [x, y, b])
        .collect()
}

/// Given line a -- b -- c constructs a rounded outer corner at point b
fn elbow(a: Point, b: Point, c: Point, width: f64) -> Vec<Point> {
    let perp_ab = Rotation2::new(FRAC_PI_2) * (b - a).normalize() * width;
    let perp_bc = Rotation2::new(FRAC_PI_2) * (c - b).normalize() * width;

    if ccw_turn(a, b, c) {
        arc(b - perp_ab, b, b - perp_bc)
    } else {
        arc(b + perp_bc, b, b + perp_ab)
    }
}

/// Check if shortest rotation from `from` to `to` is counterclockwise
fn ccw(from: &Vector, to: &Vector) -> bool { 
    from.perp(to) > 0.0
}

/// Check if line `start` -- `through` -- `end` turns counterclockwise
fn ccw_turn(start: Point, through: Point, end: Point) -> bool {
    return ccw(&(through - start), &(end - through));
}

/// Angle from `from` to `to` in counterclockwise direction
fn ccw_angle(from: &Vector, to: &Vector) -> f64 {
    if ccw(from, to) {
        from.angle(to)
    } else {
        TAU - from.angle(to)
    }
}

#[cfg(test)]
mod tests {
    mod ccw {
        #[test]
        fn acute_angle_vectors() {
            let from = Vector::new(-1.0, 2.0);
            let to = Vector::new(-2.0, 1.0);

            assert!(ccw(from, to));
            assert!(!ccw(to, from));
        }

        #[test]
        fn perpendicular_vectors() {
            let from = Vector::new(1.0, 0.0);
            let to = Vector::new(0.0, 1.0);

            assert!(ccw(from, to));
            assert!(!ccw(to, from));
        }

        #[test]
        fn obtuse_angle_vectors() {
            let from = Vector::new(3.0, 2.0);
            let to = Vector::new(-2.0, 0.0);

            assert!(ccw(from, to));
            assert!(!ccw(to, from));
        }
    }
}