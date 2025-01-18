use macromacroquad::prelude::*;
use macroquad::{
    color::{self},
    input::{is_mouse_button_pressed, mouse_position},
    math::vec2,
    shapes::draw_rectangle,
    text::draw_text,
    window::{next_frame, Conf},
};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn conf() -> Conf {
    Conf {
        window_title: "Windows demo".into(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        fullscreen: false,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    let dim = vec2(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32);

    let mut root = Window::root(dim);
    // there's no need to store the windows in variables once made,
    // they can be retrieved as `root.get("top-bar").get("button")`
    let mut top_bar = root.top_left("top-bar", (perc(80.), perc(10.)), (perc(10.), perc(0.)));
    let button = top_bar.center("button", (perc(20.), perc(50.)));
    let bottom_bar = root.bottom_right("bottom-bar", (perc(80.), perc(10.)), (perc(10.), perc(0.)));

    loop {
        root.for_xywh(|x, y, w, h| draw_rectangle(x, y, w, h, color::BLACK));
        bottom_bar.for_xywh(|x, y, w, h| draw_rectangle(x, y, w, h, color::RED));
        top_bar.for_xywh(|x, y, w, h| draw_rectangle(x, y, w, h, color::GREEN));

        {
            let (x, y, w, h) = button.xywh();
            draw_rectangle(x, y, w, h, color::BLUE);
            draw_text("Click Me", x + 10., y + 20., 20., color::BLACK);
        }

        // dynamically add/remove sub-windows
        // check if coordinate lies in window
        if is_mouse_button_pressed(macroquad::input::MouseButton::Left)
            && button.contains(mouse_position())
        {
            match root.get_opt("dialog") {
                Some(dialog) => {
                    dialog.remove();
                }
                None => {
                    root.center("dialog", perc(40.));
                }
            }
        }

        if let Some(dialog) = root.get_opt("dialog") {
            dialog.for_xywh(|x, y, w, h| draw_rectangle(x, y, w, h, color::YELLOW));
        }

        next_frame().await
    }
}
