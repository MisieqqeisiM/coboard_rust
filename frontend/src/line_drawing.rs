use common::entities::{Line, Position};
use itertools::Itertools;
use nalgebra::{Point2, Rotation2, Vector2};
use std::f64::consts::{FRAC_PI_2, TAU};

type Point = Point2<f64>;
type Vector = Vector2<f64>;

pub fn line_into_triangle_strip(line: &Line) -> Vec<Point> {
    let width = line.width as f64;
    let line = line
        .points
        .iter()
        .map(|Position { x, y }| Point::new(x.to_owned() as f64, y.to_owned() as f64))
        .collect_vec();
    if let [] = line[..] {
        Vec::new()
    } else if let [a] = line[..] {
        circle(a, width)
    } else {
        let mut result = vec![cap(line[0], line[1], width)];

        for i in 0..(line.len() - 1) {
            result.push(rectangle(line[i], line[i + 1], width));
            if i + 2 < line.len() {
                result.push(elbow(line[i], line[i + 1], line[i + 2], width));
            }
        }
        result.push(cap(line[line.len() - 1], line[line.len() - 2], width));

        let result = result.into_iter().flat_map(|v| v).collect_vec();
        [&[result[0]], &result[..], &[result[result.len() - 1]]].concat()
    }
}

fn rectangle(from: Point, to: Point, width: f64) -> Vec<Point> {
    let dir = (to - from).normalize();
    let perp = Rotation2::new(FRAC_PI_2) * dir;

    let p1 = from + perp * width;
    let p2 = from - perp * width;
    let p3 = to + perp * width;
    let p4 = to - perp * width;

    vec![p1, p2, p3, p4]
}

const ANGLE_RES: f64 = 0.3;

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
    let mut points = vec![start, a];
    let rotation = Rotation2::new(TAU / segment_count as f64);
    for _ in 0..segment_count {
        let prev = points[points.len() - 1];
        points.push(a + rotation * (prev - a));
        points.push(a);
    }
    points.push(start);
    points
}

/// Constructs arc centered in `b` ranging from point `a` to `c` in counterclockwise direction
fn arc(a: Point, b: Point, c: Point) -> Vec<Point> {
    let angle = ccw_angle(&(a - b), &(c - b));
    let segment_count = (angle / ANGLE_RES).ceil() as u32;
    if segment_count == 0 {
        return vec![];
    }
    let rotation = Rotation2::new(angle / segment_count as f64);
    let mut points = vec![a, b];
    for _ in 0..(segment_count - 1) {
        let prev = points[points.len() - 2];
        points.push(b + rotation * (prev - b));
        points.push(b);
    }
    points.push(c);
    points
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
        use crate::line_drawing::{ccw, Vector};

        #[test]
        fn acute_angle_vectors() {
            let from = Vector::new(-1.0, 2.0);
            let to = Vector::new(-2.0, 1.0);

            assert!(ccw(&from, &to));
            assert!(!ccw(&to, &from));
        }

        #[test]
        fn perpendicular_vectors() {
            let from = Vector::new(1.0, 0.0);
            let to = Vector::new(0.0, 1.0);

            assert!(ccw(&from, &to));
            assert!(!ccw(&to, &from));
        }

        #[test]
        fn obtuse_angle_vectors() {
            let from = Vector::new(3.0, 2.0);
            let to = Vector::new(-2.0, 0.0);

            assert!(ccw(&from, &to));
            assert!(!ccw(&to, &from));
        }
    }
}
