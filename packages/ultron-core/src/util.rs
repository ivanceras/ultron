use nalgebra::Point2;
use std::fmt;

pub fn cast_point(point: Point2<i32>) -> Point2<usize> {
    Point2::new(point.x.try_into().unwrap(), point.y.try_into().unwrap())
}

pub fn normalize_points<T>(p1: Point2<T>, p2: Point2<T>) -> (Point2<T>, Point2<T>)
where
    T: Copy + Clone + PartialEq + fmt::Debug + Ord + 'static,
{
    let min_x = p1.x.min(p2.x);
    let min_y = p1.y.min(p2.y);
    let max_x = p1.x.max(p2.x);
    let max_y = p1.y.max(p2.y);
    (Point2::new(min_x, min_y), Point2::new(max_x, max_y))
}

#[allow(clippy::if_same_then_else)]
#[allow(clippy::comparison_chain)]
/// order the point by y (top is first) and then by x (left is first)
pub fn reorder_top_down_left_right<T>(p1: Point2<T>, p2: Point2<T>) -> (Point2<T>, Point2<T>)
where
    T: Copy + Clone + PartialEq + fmt::Debug + Ord + 'static,
{
    if p1.y == p2.y {
        if p1.x == p2.x {
            (p1, p2)
        } else if p1.x < p2.x {
            (p1, p2)
        } else if p2.x < p1.x {
            (p2, p1)
        } else {
            unreachable!()
        }
    } else if p1.y < p2.y {
        (p1, p2)
    } else if p2.y < p1.y {
        (p2, p1)
    } else {
        unreachable!()
    }
}
