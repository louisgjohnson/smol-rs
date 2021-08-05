use hecs::{Entity, World};

use crate::{
    components::Physics,
    core::{
        get_window_size, queue_text, queue_text_ex, render_rect, render_text_queue, reset_offset,
        set_offset,
    },
    math::{Rectangle, Vec2},
    render::{Color, GREEN, RED, WHITE},
    text_render::{TextAlignment, TextQueueConfig},
};

use super::line_border;

pub struct SideBar {}

impl SideBar {
    pub fn render(&self, world: &World, player: Entity) {
        let window_size: Vec2 = get_window_size().into();
        let y = 5.;
        let height = window_size.y - y;
        let width = 300.;
        let x = window_size.x - width;
        set_offset(Vec2 { x, y });
        render_rect(0., 0., width, height, Color(28, 33, 43, 1.));
        let healthbar_width = width - 20.;
        let physics = world.get::<Physics>(player).unwrap();
        render_rect(10., 10., healthbar_width, 30., RED);
        render_rect(
            10.,
            10.,
            healthbar_width * (physics.health as f32 / physics.max_health as f32),
            30.,
            GREEN,
        );
        queue_text_ex(
            &format!("{}/{}", &physics.health, &physics.max_health),
            TextQueueConfig {
                position: Vec2::new(healthbar_width / 2. + 10., 10.),
                color: WHITE,
                font_size: 14.,
                horizontal_alginment: TextAlignment::Center,
            },
        );

        render_text_queue();

        reset_offset();

        let rect = Rectangle {
            x,
            y,
            w: width,
            h: height,
        };
        line_border(rect, 5.0, Color(255, 255, 255, 1.));
    }
}