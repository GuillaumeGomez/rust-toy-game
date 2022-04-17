use crate::{GetDimension, GetPos};

pub fn compute_distance<A, B>(a: &A, b: &B) -> f32
where
    A: GetPos + GetDimension,
    B: GetPos + GetDimension,
{
    let a_width = a.width() as f32;
    let a_height = a.height() as f32;
    let b_width = b.width() as f32;
    let b_height = b.height() as f32;

    let a_x = a.x();
    let a_y = a.y();
    let b_x = b.x();
    let b_y = b.y();

    // FIXME: sometimes, the sprites have some "hidden" pixels (invisible ones) which are
    // accounted into the width and therefore are making the distance shorter than it really is.
    // It can be seen when a skeleton is on the left of the player: without this adjustment, its
    // attacks won't reach the player.
    let x = if b_x >= a_x + a_width {
        b_x - a_x - a_width
    } else if a_x >= b_x + b_width {
        b_x + b_width - a_x
    } else {
        b_x - a_x
    };
    let y = if b_y + b_height <= a_y {
        a_y - b_y - b_height
    } else if a_y + a_height <= b_y {
        b_y - a_y - a_height
    } else {
        b_y - a_y
    };
    (x * x + y * y).sqrt()
}

/// Returns a tuple containing: `(distance, incr X axis, incr Y axis)`.
// pub fn get_axis_distance<A, B>(a: &A, b: &B) -> (u32, i8, i8)
// where
//     A: GetPos + GetDimension,
//     B: GetPos + GetDimension,
// {
//     let x1 = a.x();
//     let x1b = x1 + a.width() as f32;
//     let y1 = a.y();
//     let y1b = y1 + a.height() as f32;
//     let x2 = b.x();
//     let x2b = x2 + b.width() as f32;
//     let y2 = b.y();
//     let y2b = y2 + b.height() as f32;

//     let left = x2b < x1;
//     let right = x1b < x2;
//     let bottom = y2b < y1;
//     let top = y1b < y2;

//     let (a, b, c) = if top && left {
//         (compute_distance(&(x1, y1b), &(x2b, y2)), -1, 1)
//     } else if left && bottom {
//         (compute_distance(&(x1, y1), &(x2b, y2b)), -1, -1)
//     } else if bottom && right {
//         (compute_distance(&(x1b, y1), &(x2, y2b)), 1, -1)
//     } else if right && top {
//         (compute_distance(&(x1b, y1b), &(x2, y2)), 1, 1)
//     } else if left {
//         ((x1 - x2b) as i32, -1, 0)
//     } else if right {
//         ((x2 - x1b) as i32, 1, 0)
//     } else if bottom {
//         ((y1 - y2b) as i32, 0, -1)
//     } else if top {
//         ((y2 - y1b) as i32, 0, 1)
//     } else {
//         // rectangles intersect
//         (0, 0, 0)
//     };
//     (a.abs() as u32, b, c)
// }

#[macro_export]
macro_rules! debug_enemy {
    ($($x:tt)*) => (
        #[cfg(feature = "debug_enemy")]
        {
            println!($($x)*);
        }
        #[cfg(not(feature = "debug_enemy"))]
        {
        }
    )
}
