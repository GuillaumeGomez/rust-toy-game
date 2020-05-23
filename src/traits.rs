
pub trait GetPos {
    fn x(&self) -> i64;
    fn y(&self) -> i64;
}

impl GetPos for (i64, i64) {
    fn x(&self) -> i64 {
        self.0
    }
    fn y(&self) -> i64 {
        self.1
    }
}

impl GetPos for &(i64, i64) {
    fn x(&self) -> i64 {
        self.0
    }
    fn y(&self) -> i64 {
        self.1
    }
}

pub trait GetDimension {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

impl GetDimension for (i64, i64) {
    fn width(&self) -> u32 { 0 }
    fn height(&self) -> u32 { 0 }
}

impl GetDimension for &(i64, i64) {
    fn width(&self) -> u32 { 0 }
    fn height(&self) -> u32 { 0 }
}
