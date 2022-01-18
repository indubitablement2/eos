use crate::client_metascape::Metascape;
use crate::constants::COLOR_ALICE_BLUE;
use crate::util::SetAlpha;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(Debug)]
pub struct DebugInfos {
    font: Ref<Font>,

    time_dilations: [f32; 300],
    time_dilations_cursor: usize,
}
impl DebugInfos {
    pub fn new() -> Self {
        let control = Control::new();

        Self {
            font: control.get_font("font", "").unwrap(),
            time_dilations: [1.0; 300],
            time_dilations_cursor: 0,
        }
    }

    pub fn update(&mut self, metascape: &Metascape) {
        // Debug time multiplier.
        self.time_dilations[self.time_dilations_cursor] = metascape.time_dilation;
        self.time_dilations_cursor = (self.time_dilations_cursor + 1) % self.time_dilations.len();
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
        control.draw_multiline(PoolArray::from_iter(iter), COLOR_ALICE_BLUE.with_alpha(0.3), 1.0, false);

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
            Vector2::new(size.x * 0.5, font_height),
            format!(
                "avg: {:.2}",
                self.time_dilations
                    .iter()
                    .fold(0.0, |acc, x| x.mul_add(1.0 / self.time_dilations.len() as f32, acc))
            ),
            COLOR_ALICE_BLUE.with_alpha(0.8),
            -1,
        );
    }
}
