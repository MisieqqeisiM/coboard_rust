use itertools::Itertools;
use leptos::html::li;
use std::f64::consts::{FRAC_PI_2, PI};

use nalgebra::{ComplexField, Matrix2, Point2, Rotation, Rotation2, Vector2};

type Point = Point2<f64>;

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

fn cap(from: Point, to: Point, width: f64) -> Vec<Point> {
    let dir = (to - from).normalize();
    let perp = Rotation2::new(FRAC_PI_2) * dir;

    let p1 = from + perp * width;
    let p2 = from - perp * width;

    arc(p1, from, p2)
}

fn circle(a: Point, width: f64) -> Vec<Point> {
    let no_segments: u32 = (2.0 * PI / ANGLE_RES).ceil() as u32;
    let start = a + Vector2::new(width, 0.0);
    let mut points = vec![start];
    let rotation = Rotation2::new(2.0 * PI / no_segments as f64);
    for _ in 0..no_segments {
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

fn arc(a: Point, b: Point, c: Point) -> Vec<Point> {
    let angle = (a - b).angle(&(c - b));
    if angle < 0.001 {
        return vec![];
    }
    let no_segments: u32 = (angle.abs() / ANGLE_RES).ceil() as u32;
    let rotation = Rotation2::new(angle / no_segments as f64);
    let mut points = vec![a];
    for _ in 0..(no_segments - 1) {
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

fn elbow(a: Point, b: Point, c: Point, width: f64) -> Vec<Point> {
    let perp_ab = Rotation2::new(FRAC_PI_2) * (b - a).normalize() * width;
    let perp_bc = Rotation2::new(FRAC_PI_2) * (c - b).normalize() * width;

    if (b - a).dot(&perp_bc) > 0.0 {
        arc(b + perp_ab, b, b + perp_bc)
    } else {
        arc(b - perp_ab, b, b - perp_bc)
    }
}