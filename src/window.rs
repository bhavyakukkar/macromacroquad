use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use macroquad::math::{vec2, Vec2};

use crate::utils::Coord;

fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<T> {
    lock.read()
        .expect("rwlock poisoned. we're fucked. can't read")
}
fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<T> {
    lock.write()
        .expect("rwlock poisoned. we're fucked. can't write")
}

pub struct Percentage(f32);

pub fn perc(value: f32) -> Percentage {
    assert!(value >= 0. && value <= 100., "invalid percentage value");
    Percentage(value)
}

pub trait ScaleInto<T> {
    fn scale_into(self, older: T) -> T;
}

impl ScaleInto<Vec2> for Percentage {
    fn scale_into(self, older: Vec2) -> Vec2 {
        Vec2 {
            x: (older.x * self.0) / 100.,
            y: (older.y * self.0) / 100.,
        }
    }
}

impl ScaleInto<Vec2> for (Percentage, Percentage) {
    fn scale_into(self, older: Vec2) -> Vec2 {
        Vec2 {
            x: (older.x * (self.0).0) / 100.,
            y: (older.y * (self.1).0) / 100.,
        }
    }
}

impl ScaleInto<Vec2> for Vec2 {
    fn scale_into(self, _older: Vec2) -> Vec2 {
        self
    }
}

#[derive(Debug)]
struct WindowInner {
    name: Option<String>,
    size_tl: Vec2,
    size_br: Vec2,
    sub_windows: RwLock<HashMap<String, Arc<RwLock<WindowInner>>>>,
    parent: Option<Arc<RwLock<WindowInner>>>,
}

impl WindowInner {
    fn new(offset_top_left: Vec2, offset_bottom_right: Vec2) -> Self {
        Self {
            name: None,
            size_tl: offset_top_left,
            size_br: offset_bottom_right,
            sub_windows: RwLock::new(HashMap::new()),
            parent: None,
        }
    }

    fn size(&self) -> Vec2 {
        self.size_br - self.size_tl
    }

    fn corners(&self) -> (Vec2, Vec2) {
        if let Some(ref parent) = self.parent {
            // this is some child window
            let parent = read_lock(&parent);
            let (parent_size_tl, _) = parent.corners();
            (parent_size_tl + self.size_tl, parent_size_tl + self.size_br)
        } else {
            // this is the root window
            assert!(
                self.size_tl.x == 0. && self.size_tl.y == 0.,
                "DEV top_left should be (0,0) as this is the root window"
            );
            (self.size_tl, self.size_br)
        }
    }

    fn corners_offset(&self) -> (Vec2, Vec2) {
        if let Some(ref parent) = self.parent {
            // this is some child window
            let parent = read_lock(&parent);
            let parent_corners = parent.corners_offset();
            (
                parent_corners.0 + self.size_tl,
                parent_corners.1 + ((parent.size_br - parent.size_tl) - self.size_br),
            )
        } else {
            // this is the root window
            assert!(
                self.size_tl.x == 0. && self.size_tl.y == 0.,
                "DEV top_left should be (0,0) as this is the root window"
            );
            (self.size_tl, Vec2::ZERO)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Window(Arc<RwLock<WindowInner>>);

impl Window {
    pub fn contains(&self, coord: impl Coord<f32>) -> bool {
        let x = coord.x();
        let y = coord.y();
        let (size_tl, size_br) = self.corners();
        x >= size_tl.x && x <= size_br.x && y >= size_tl.y && y <= size_br.y
    }

    pub fn root(size: Vec2) -> Self {
        Window(Arc::new(RwLock::new(WindowInner::new(vec2(0., 0.), size))))
    }

    pub fn corners(&self) -> (Vec2, Vec2) {
        read_lock(&self.0).corners()
    }

    pub fn for_corners<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(Vec2, Vec2) -> R,
    {
        let (size_tl, size_br) = self.corners();
        f(size_tl, size_br)
    }

    pub fn corners_offset(&self) -> (Vec2, Vec2) {
        read_lock(&self.0).corners_offset()
    }

    pub fn for_corners_offset<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(Vec2, Vec2) -> R,
    {
        let (size_tl, size_br) = self.corners_offset();
        f(size_tl, size_br)
    }

    pub fn coords(&self) -> (f32, f32, f32, f32) {
        let (size_tl, size_br) = self.corners();
        (size_tl.x, size_tl.y, size_br.x, size_br.y)
    }

    pub fn for_coords<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(f32, f32, f32, f32) -> R,
    {
        let (size_tl, size_br) = self.corners();
        f(size_tl.x, size_tl.y, size_br.x, size_br.y)
    }

    pub fn xywh(&self) -> (f32, f32, f32, f32) {
        let (size_tl, size_br) = self.corners();
        (
            size_tl.x,
            size_tl.y,
            size_br.x - size_tl.x,
            size_br.y - size_tl.y,
        )
    }

    pub fn for_xywh<F, R>(&self, mut f: F) -> R
    where
        F: FnMut(f32, f32, f32, f32) -> R,
    {
        let (size_tl, size_br) = self.corners();
        f(
            size_tl.x,
            size_tl.y,
            size_br.x - size_tl.x,
            size_br.y - size_tl.y,
        )
    }

    pub fn size(&self) -> Vec2 {
        read_lock(&self.0).size()
    }

    fn add_sub_window(&mut self, name: impl Into<String>, size_tl: Vec2, size_br: Vec2) -> Window {
        let name = name.into();
        let sub_window = Arc::new(RwLock::new(WindowInner {
            name: Some(name.clone()),
            size_tl,
            size_br,
            sub_windows: RwLock::new(HashMap::new()),
            parent: Some(self.0.clone()),
        }));

        let sub_windows = &read_lock(&self.0).sub_windows;
        _ = write_lock(&sub_windows)
            .insert(name, sub_window.clone())
            .map(|old_sub_window| {
                {
                    let mut old_sub_window = write_lock(&old_sub_window);
                    old_sub_window.parent = None;
                }
                Window(old_sub_window).remove();
            });
        Window(sub_window)
    }

    pub fn top_left(
        &mut self,
        name: impl Into<String>,
        size: impl ScaleInto<Vec2>,
        offset_top_left: impl ScaleInto<Vec2>,
    ) -> Window {
        let self_size = self.size();

        let off_tl = offset_top_left.scale_into(self_size);
        let size = size.scale_into(self_size);

        assert!(
            off_tl.x < self_size.x && off_tl.y < self_size.y,
            "top-left offset of child-window cannot be larger than parent window size"
        );
        assert!(
            (off_tl.x + size.x) < self_size.x && (off_tl.y + size.y) < self_size.y,
            "child-window will extend past parent window on the bottom-right corner"
        );

        self.add_sub_window(name, off_tl, off_tl + size)
    }

    pub fn bottom_right(
        &mut self,
        name: impl Into<String>,
        size: impl ScaleInto<Vec2>,
        offset_bottom_right: impl ScaleInto<Vec2>,
    ) -> Window {
        let self_size = self.size();

        let off_br = offset_bottom_right.scale_into(self_size);
        let size = size.scale_into(self_size);

        assert!(
            off_br.x < self_size.x && off_br.y < self_size.y,
            "bottom-right offset of child-window cannot be larger than parent window size"
        );
        assert!(
            (off_br.x + size.x) < self_size.x && (off_br.y + size.y) < self_size.y,
            "child-window will extend past parent window on the top-left corner"
        );

        self.add_sub_window(name, self_size - (size + off_br), self_size - off_br)
    }

    pub fn center(&mut self, name: impl Into<String>, size: impl ScaleInto<Vec2>) -> Window {
        let self_size = self.size();
        let size = size.scale_into(self_size);
        assert!(
            size.x < self_size.x && size.y < self_size.y,
            "child-window cannot be larger than parent-window"
        );

        let size_tl = vec2((self_size.x - size.x) / 2., (self_size.y - size.y) / 2.);
        self.add_sub_window(name, size_tl, size_tl + size)
    }

    pub fn get_opt(&self, name: &str) -> Option<Self> {
        read_lock(&read_lock(&self.0).sub_windows)
            .get(name)
            .map(|inner| Window(inner.clone()))
    }

    pub fn get(&self, name: &str) -> Self {
        self.get_opt(name)
            .expect(&format!("no sub-window found with name '{name}'"))
    }

    pub fn remove(self) {
        {
            let this_window = read_lock(&self.0);
            if let Some(parent) = &this_window.parent {
                let parent = write_lock(&parent);
                let mut brother_windows = write_lock(&parent.sub_windows);
                drop(
                    brother_windows
                        .remove(this_window.name.as_ref().expect("api doesn't allow this"))
                        .expect("api shouldn't allow this"),
                );
            }
        }

        _ = Arc::into_inner(self.0)
            .expect("api doesn't allow this")
            .into_inner()
            .expect("rwlock poisoned. we're fucked. can't get inner");
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    use macroquad::math::{vec2, Vec2};

    use super::{perc, Window};

    #[test]
    fn sub_windows() {
        let mut root = Window::root(vec2(1920., 1080.));
        assert_eq!(root.size(), vec2(1920., 1080.));
        assert_eq!(root.corners(), (vec2(0., 0.), vec2(1920., 1080.)));
        assert_eq!(root.corners_offset(), (vec2(0., 0.), vec2(0., 0.)));

        let mut dialog = root.top_left("dialog", perc(50.), perc(25.));
        assert_eq!(dialog.size(), vec2(960., 540.));
        assert_eq!(dialog.corners(), (vec2(480., 270.), vec2(1440., 810.)));
        assert_eq!(
            dialog.corners_offset(),
            (vec2(480., 270.), vec2(480., 270.))
        );

        let mut button = dialog.top_left("button", perc(50.), perc(25.));
        assert_eq!(button.size(), vec2(480., 270.));
        assert_eq!(button.corners(), (vec2(720., 405.), vec2(1200., 675.)));
        assert_eq!(
            button.corners_offset(),
            (vec2(720., 405.), vec2(720., 405.))
        );

        let mut text = button.bottom_right("text", perc(50.), Vec2::ZERO);
        assert_eq!(text.size(), vec2(240., 135.));
        assert_eq!(text.corners(), (vec2(960., 540.), vec2(1200., 675.)));
        assert_eq!(text.corners_offset(), (vec2(960., 540.), vec2(720., 405.)));

        // should panic because child-window will extend past parent window `text` on the top-left corner
        assert!(catch_unwind(move || text.bottom_right("", perc(50.), perc(50.))).is_err());
    }
}
