use crate::{GetDimension, GetPos};

pub fn compute_distance<A, B>(a: &A, b: &B) -> i32
where
    A: GetPos + GetDimension,
    B: GetPos + GetDimension,
{
    let a_width = a.width() as i64;
    let a_height = a.height() as i64;
    let b_width = b.width() as i64;
    let b_height = b.height() as i64;
    let mut x;
    let mut y;

    if a_width == 0 || b_width == 0 {
        x = b.x() - a.x();
        y = b.y() - a.y();
    } else {
        let a_x = a.x() as i64;
        let a_y = a.y() as i64;
        let b_x = b.x() as i64;
        let b_y = b.y() as i64;

        if b_x >= a_x + a_width {
            x = b_x - a_x - a_width;
        } else if a_x >= b_x + b_width {
            x = b_x + b_width - a_x;
        } else {
            x = b_x - a_x;
        }
        if b_y >= a_y + a_height {
            y = b_y - a_y - a_height;
        } else if a_y >= b_y + b_height {
            y = b_y + b_height - a_y;
        } else {
            y = b_y - a_y;
        }
    }
    x *= x;
    y *= y;
    ((x + y) as f32).sqrt() as i32
}
