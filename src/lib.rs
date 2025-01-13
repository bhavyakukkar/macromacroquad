pub mod window;

pub mod prelude {
    pub use crate::window::{p, ScaleInto, Window};
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

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
