use nalgebra::Point2;

pub(crate) fn cast_point(point: Point2<i32>) -> Point2<usize> {
    Point2::new(point.x.try_into().unwrap(), point.y.try_into().unwrap())
}
