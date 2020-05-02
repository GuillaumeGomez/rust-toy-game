use crate::GetPos;

pub fn compute_distance<A, B>(a: &A, b: &B) -> i32
where
    A: GetPos,
    B: GetPos,
{
    let mut x = b.x() - a.x();
    let mut y = b.y() - a.y();
    x *= x;
    y *= y;
    ((x + y) as f32).sqrt() as i32
}
