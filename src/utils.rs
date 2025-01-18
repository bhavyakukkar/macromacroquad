use macroquad::math::Vec2;

pub trait Coord<T>: Copy {
    fn x(self) -> T;
    fn y(self) -> T;
}

impl<T: Copy> Coord<T> for (T, T) {
    fn x(self) -> T {
        self.0
    }

    fn y(self) -> T {
        self.1
    }
}

impl Coord<f32> for Vec2 {
    fn x(self) -> f32 {
        self.x
    }

    fn y(self) -> f32 {
        self.y
    }
}

impl<T: Copy> Coord<T> for [T; 2] {
    fn x(self) -> T {
        self[0]
    }

    fn y(self) -> T {
        self[1]
    }
}
