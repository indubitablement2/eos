use crate::client_metascape::Metascape;
use crate::constants::COLOR_ALICE_BLUE;
use crate::util::SetAlpha;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(Debug)]
pub struct DebugInfos {
    font: Ref<Font>,

    current_tick_buffer: i64,
    target_tick_buffer: i64,

    time_dilations: [f32; 300],
    time_dilations_cursor: usize,
}
impl DebugInfos {
    pub fn new() -> Self {
        let control = Control::new();
        let font = control.get_font("font", "").unwrap();
        control.queue_free();

        Self {
            font,
            current_tick_buffer: 1,
            target_tick_buffer: 1,
            time_dilations: [1.0; 300],
            time_dilations_cursor: 0,
        }
    }

    pub fn update(&mut self, metascape: &Metascape) {
        // Tick buffer.
        self.current_tick_buffer = metascape.current_tick_buffer;
        self.target_tick_buffer = metascape.target_tick_buffer as i64;

        // Debug time multiplier.
        self.time_dilations[self.time_dilations_cursor] = metascape.time_dilation;
        self.time_dilations_cursor = (self.time_dilations_cursor + 1) % self.time_dilations.len();
    }

    pub unsafe fn draw_tick_buffer(&self, control: Ref<Control>) {
        let control = control.assume_safe();
        let size = control.size();

        let font = self.font.assume_safe();
        let font_height = font.get_height() as f32;
        let font_width = font.get_char_size(65, 0).x;

        let graph_rect = Rect2 {
            position: Vector2::new(font_width * 3.0, font_height * 3.0 + 2.0),
            size: Vector2::new(size.x - font_width * 6.0, size.y - font_height * 3.0 - 2.0),
        };

        let graph_min = self.target_tick_buffer - 6;
        let graph_max = self.target_tick_buffer + 6;
        let max_point_y = (graph_max - graph_min) as f32;

        // Draw graph bound.
        control.draw_polyline(PoolArray::from_slice(&[
            graph_rect.position,
            Vector2::new(graph_rect.position.x + graph_rect.size.x, graph_rect.position.y),
            Vector2::new(graph_rect.position.x + graph_rect.size.x, graph_rect.position.y + graph_rect.size.y),
            Vector2::new(graph_rect.position.x, graph_rect.position.y + graph_rect.size.y),
            graph_rect.position
        ]), COLOR_ALICE_BLUE.with_alpha(0.3), 1.0, false);

        // Draw current buffer.
        let uy = (self.current_tick_buffer - graph_min) as f32 / max_point_y;
        let y = uy.mul_add(graph_rect.size.y, graph_rect.position.y);
        control.draw_line(
            Vector2::new(graph_rect.position.x, y),
            Vector2::new(graph_rect.position.x + graph_rect.size.x, y),
            COLOR_ALICE_BLUE.with_alpha(0.5),
            1.0,
            false,
        );
        control.draw_string(
            font,
            Vector2::new(0.0, y + 4.0),
            "C->",
            COLOR_ALICE_BLUE.with_alpha(0.5),
            -1,
        );

        // Draw target buffer.
        let y = graph_rect.size.y.mul_add(0.5, graph_rect.position.y);
        control.draw_line(
            Vector2::new(graph_rect.position.x, y),
            Vector2::new(graph_rect.position.x + graph_rect.size.x, y),
            COLOR_ALICE_BLUE.with_alpha(0.5),
            1.0,
            false,
        );
        control.draw_string(
            font,
            Vector2::new(graph_rect.position.x + graph_rect.size.x, y + 4.0),
            "<-T",
            COLOR_ALICE_BLUE.with_alpha(0.5),
            -1,
        );

        // Draw text.
        control.draw_string(
            font,
            Vector2::new(0.0, font_height),
            format!(
                "target buffer: {:2}",
                self.target_tick_buffer
            ),
            COLOR_ALICE_BLUE.with_alpha(0.8),
            -1,
        );
        control.draw_string(
            font,
            Vector2::new(0.0, font_height * 2.0 + 2.0),
            format!(
                "current buffer: {:2}",
                self.current_tick_buffer
            ),
            COLOR_ALICE_BLUE.with_alpha(0.8),
            -1,
        );
    }

    pub unsafe fn draw_time_dilation(&self, control: Ref<Control>) {
        let control = control.assume_safe();
        let size = control.size();

        let font = self.font.assume_safe();
        let font_height = font.get_height() as f32;

        let graph_rect = Rect2 {
            position: Vector2::new(0.0, font_height + 2.0),
            size: Vector2::new(size.x, size.y - font_height * 2.0 - 4.0),
        };

        let (min, max) = self
            .time_dilations
            .iter()
            .fold((f32::MAX, f32::MIN), |(min, max), x| (min.min(*x), max.max(*x)));
        let graph_min = min.div_euclid(0.1) * 0.1;
        let graph_max = max.div_euclid(0.1).mul_add(0.1, 0.1);

        // Draw lines.
        let num_line = 5;
        let iter = (0..num_line).into_iter().flat_map(|i| {
            let p = i as f32 / (num_line - 1) as f32;
            let y = graph_rect.size.y * p;

            [
                Vector2::new(graph_rect.position.x, graph_rect.position.y + y),
                Vector2::new(graph_rect.size.x, graph_rect.position.y + y),
            ]
        });
        control.draw_multiline(PoolArray::from_iter(iter), COLOR_ALICE_BLUE.with_alpha(0.3), 1.0, true);

        // Draw graph.
        let dif = graph_max - graph_min;
        let iter = (self.time_dilations_cursor..self.time_dilations.len())
            .into_iter()
            .chain(0..self.time_dilations_cursor)
            .zip(0..)
            .map(|(i, num)| {
                Vector2::new(
                    (num as f32 / self.time_dilations.len() as f32).mul_add(graph_rect.size.x, graph_rect.position.x),
                    (((self.time_dilations[i] - graph_min) / dif) - 1.0)
                        .abs()
                        .mul_add(graph_rect.size.y, graph_rect.position.y),
                )
            });
        control.draw_polyline(PoolArray::from_iter(iter), COLOR_ALICE_BLUE.with_alpha(0.5), 2.0, true);

        // Draw graph min/max numbers.
        control.draw_string(
            font,
            Vector2::new(0.0, font_height),
            format!("{:.2}", graph_max),
            COLOR_ALICE_BLUE.with_alpha(0.8),
            -1,
        );
        control.draw_string(
            font,
            Vector2::new(0.0, size.y),
            format!("{:.2}", graph_min),
            COLOR_ALICE_BLUE.with_alpha(0.8),
            -1,
        );

        // Draw average number.
        control.draw_string(
            font,
            Vector2::new(size.x - 6.0 * 24.0, font_height),
            format!(
                "avg. time dilation: {:.2}",
                self.time_dilations
                    .iter()
                    .fold(0.0, |acc, x| x.mul_add(1.0 / self.time_dilations.len() as f32, acc))
            ),
            COLOR_ALICE_BLUE.with_alpha(0.8),
            -1,
        );
    }
}
