use darkkit::{prelude::*, window};
use macroquad::{
    color::{self, Color},
    input::{is_mouse_button_pressed, mouse_position},
    math::{vec2, Vec2},
    shapes,
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

fn draw_rectangle((corner_top_left, corner_bottom_right): (Vec2, Vec2), color: Color) {
    let Vec2 { x, y } = corner_top_left;
    let Vec2 { x: u, y: v } = corner_bottom_right;
    shapes::draw_rectangle(x, y, u - x, v - y, color);
}

#[macroquad::main(conf)]
async fn main() {
    let dim = vec2(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32);

    let mut root = window! { s:
        dim,
        w: [
            ("top-bar", window! {
                s: dim * vec2(0.8, 0.1),
                o: dim * vec2(0.1, 0.), w: [
                    ("button", window! {
                        s: dim * vec2(0.8, 1.) * vec2(0.2, 0.05),
                        o: dim * vec2(0.8, 0.0) * vec2(0.4, 0.),
                    })
                ]}),
            ("bottom-bar", window! {
                s: dim * vec2(0.8, 0.1),
                o: dim * vec2(0.1, 0.9),
            })
        ]
    };
    let mut dialog: Option<Window> = None;

    loop {
        draw_rectangle(root.corners(), color::BLACK);

        draw_rectangle(root.get("bottom-bar").corners(), color::RED);

        {
            let top_bar = root.get("top-bar");
            draw_rectangle(top_bar.corners(), color::GREEN);

            let button = top_bar.get("button");
            let (top_left, bottom_right) = button.corners();
            draw_rectangle(button.corners(), color::BLUE);
            draw_text(
                "Click Me",
                top_left.x + 10.,
                top_left.y + 20.,
                20.,
                color::BLACK,
            );

            // dynamically add/remove sub-windows
            if is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                let mouse_pos = mouse_position();
                if (mouse_pos.0 >= top_left.x && mouse_pos.0 <= bottom_right.x)
                    && (mouse_pos.1 >= top_left.y && mouse_pos.1 <= bottom_right.y)
                {
                    dialog = match dialog {
                        Some(dialog) => {
                            dialog.remove();
                            None
                        }
                        None => Some(root.sub_window(
                            "dialog",
                            Some(dim * vec2(0.4, 0.4)),
                            Some(dim * vec2(0.3, 0.3)),
                            None,
                        )),
                    };
                }
            }

            if let Some(dialog) = &dialog {
                draw_rectangle(dialog.corners(), color::YELLOW);
            }
        }

        next_frame().await
    }
}
