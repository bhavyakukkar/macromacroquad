pub mod utils;

// TODO fns top_right, bottom_left, center
// TODO include some threshold before panicking in sub-window fns if difference is negligible
//      (at the moment doing top_left(perc(50.), perc(50.)) panics)
// TODO you can also report names of problematic windows when asserting
// TODO add elevation (zIndex) to WindowInner
pub mod window;

pub mod prelude {
    pub use crate::window::{perc, ScaleInto, Window};
}

#[macro_export]
macro_rules! window {
    (s: $size:expr, o: $off_tl:expr, w: [$($sub_window:expr),+ $(,)?]) => {{
        let mut root = $crate::window::Window(std::sync::Arc::new(std::sync::RwLock::new(
            $crate::window::WindowInner::new(Some($size), Some($off_tl), None)
        )));
        for (name, window) in [$($sub_window),+] {
            root.add_sub_window(name, window);
        }
        root
    }};

    (s: $size:expr, w: [$($sub_window:expr),+ $(,)?]) => {{
        let mut root = $crate::window::Window(std::sync::Arc::new(std::sync::RwLock::new(
            $crate::window::WindowInner::new(Some($size), None, None)
        )));
        for (name, window) in [$($sub_window),+] {
            root.add_sub_window(name, window);
        }
        root
    }};

    (s: $size:expr, o: $off_tl:expr $(,)?) => {{
        $crate::window::Window(std::sync::Arc::new(std::sync::RwLock::new(
            $crate::window::WindowInner::new(Some($size), Some($off_tl), None)
        )))
    }};

    (s: $size:expr $(,)?) => {{
        $crate::window::Window(std::sync::Arc::new(std::sync::RwLock::new(
            $crate::window::WindowInner::new(Some($size), None, None)
        )))
    }};
}
