use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use macroquad::math::{vec2, Vec2};

fn read_lock<T>(lock: &RwLock<T>) -> RwLockReadGuard<T> {
    lock.read()
        .expect("rwlock poisoned. we're fucked. can't read")
}
fn write_lock<T>(lock: &RwLock<T>) -> RwLockWriteGuard<T> {
    lock.write()
        .expect("rwlock poisoned. we're fucked. can't write")
}

#[derive(Debug)]
pub struct WindowInner {
    name: Option<String>,
    off_tl: Vec2,
    off_br: Vec2,
    sub_windows: RwLock<HashMap<String, Arc<RwLock<WindowInner>>>>,
    parent: Option<Arc<RwLock<WindowInner>>>,
}

pub struct Percentage(f32);

pub fn p(value: f32) -> Percentage {
    assert!(value >= 0. && value <= 100., "invalid percentage value");
    Percentage(value)
}

pub trait ScaleInto<T> {
    fn scale_into(self, older: T) -> T;
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

impl WindowInner {
    pub fn new(
        size: Option<Vec2>,
        offset_top_left: Option<Vec2>,
        offset_bottom_right: Option<Vec2>,
    ) -> Self {
        let size = size.unwrap_or(vec2(0., 0.));
        let off_tl = offset_top_left.unwrap_or(vec2(0., 0.));
        WindowInner {
            name: None,
            off_tl,
            off_br: offset_bottom_right.unwrap_or(size + off_tl),
            sub_windows: RwLock::new(HashMap::new()),
            parent: None,
        }
    }

    pub fn seal(&mut self) {
        let sub_windows = read_lock(&self.sub_windows);
        for (name, window) in sub_windows.iter() {
            let mut window = write_lock(&window);
            let new_off_br = (self.off_br - self.off_tl) - window.off_br;
            assert!(
                new_off_br.x > 0. && new_off_br.y > 0.,
                "invalid dimensions for sub-window '{name}'",
            );
            window.off_br = new_off_br;
        }
    }

    pub fn size(&self) -> Vec2 {
        self.off_br - self.off_tl
    }
}

// impl Default for WindowInner {
//     fn default() -> Self {
//         WindowInner {
//             name: None,
//             off_tl: vec2(0., 0.),
//             off_br: vec2(0., 0.),
//             sub_windows: RwLock::new(HashMap::new()),
//         }
//     }
// }

#[derive(Clone, Debug)]
pub struct Window(pub Arc<RwLock<WindowInner>>);

impl Window {
    pub fn root(size: Vec2) -> Self {
        Window(Arc::new(RwLock::new(WindowInner::new(
            Some(size),
            None,
            None,
        ))))
    }

    pub fn corners(&self) -> (Vec2, Vec2) {
        (read_lock(&self.0).off_tl, read_lock(&self.0).off_br)
    }

    pub fn size(&self) -> Vec2 {
        read_lock(&self.0).size()
    }

    pub fn add_sub_window(&mut self, name: impl Into<String>, window: Self) {
        let name = name.into();
        {
            let mut window = write_lock(&window.0);
            let new_off_br = read_lock(&self.0).size() - window.off_br;
            assert!(
                new_off_br.x >= 0. && new_off_br.y >= 0.,
                "invalid dimensions for sub-window '{name}'",
            );
            let (off_tl, _) = self.corners();
            window.off_tl += off_tl;
            window.off_br += off_tl;

            window.name = Some(name.clone());
            window.parent = Some(self.0.clone());
        }

        let sub_windows = &read_lock(&self.0).sub_windows;
        write_lock(&sub_windows)
            .insert(name, window.0)
            .map(|old_sub_window| {
                {
                    let mut old_sub_window = write_lock(&old_sub_window);
                    old_sub_window.parent = None;
                }
                Window(old_sub_window).remove();
            });
    }

    pub fn sub_window(
        &mut self,
        name: impl Into<String>,
        // size: Option<impl ScaleInto<Vec2>>,
        // offset_top_left: Option<impl ScaleInto<Vec2>>,
        // offset_bottom_right: Option<impl ScaleInto<Vec2>>,
        size: Option<Vec2>,
        offset_top_left: Option<Vec2>,
        offset_bottom_right: Option<Vec2>,
    ) -> Self {
        let (self_off_tl, self_off_br) = self.corners();
        let self_size = self.size();
        let new_window = Arc::new(RwLock::new(WindowInner::new(
            size.map(|size| size.scale_into(self_size)),
            offset_top_left.map(|off_tl| off_tl.scale_into(self_off_tl)),
            offset_bottom_right.map(|off_br| off_br.scale_into(self_off_br)),
        )));
        self.add_sub_window(name, Window(new_window.clone()));
        Window(new_window)
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

    pub fn remove(self) -> WindowInner {
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

        Arc::into_inner(self.0)
            .expect("api doesn't allow this")
            .into_inner()
            .expect("rwlock poisoned. we're fucked. can't get inner")
    }

    // TODO
    pub fn for_each<F, T>(&self, f: F) -> (T, HashMap<String, (T, HashMap<String, T>)>)
    where
        F: Fn(&Self) -> T + Clone,
    {
        (
            f(self),
            read_lock(&read_lock(&self.0).sub_windows)
                .iter()
                .map(|(name, window)| (name.clone(), Window(window.clone()).for_each(f.clone())))
                .collect::<HashMap<_, _>>(),
        );
        todo!()
    }
}

// impl From<(WindowInner, HashMap<String, WindowInner>)> for Window {
//     fn from(value: (WindowInner, HashMap<String, WindowInner>)) -> Self {
//         let sub_windows = value.1.into_iter().map(|(name, window)| 2);
//         // Window (
//         //     Arc::new(value.0),
//         //     sub_windows: value.1.into_iter().map(|(name, window)| 2),
//         // )
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use crate::window;
    use macroquad::math::vec2;

    #[test]
    fn it_works() {
        let root = window! {s: vec2(1920., 1080.), w: [
            ("tool-bar", window! {s: vec2(1920., 30.), w: [
                ("panel", window! {s: vec2(960., 30.)}),
            ]}),
            ("health-bar", window! {s: vec2(1920., 30.), o: vec2(0., 1050.)}),
        ]};

        assert_eq!(format!("{root:?}"), "hey");
    }
}
