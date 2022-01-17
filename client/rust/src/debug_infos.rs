use crate::client_metascape::Metascape;
use gdnative::api::{Font, Node2D};
use gdnative::prelude::*;

#[derive(Debug)]
pub struct DebugInfos {
    time_multipliers: [f32; 300],
    time_multipliers_cursor: usize,
}
impl DebugInfos {
    pub fn new() -> Self {
        Self {
            time_multipliers: [1.0; 300],
            time_multipliers_cursor: 0,
        }
    }

    pub fn update(&mut self, metascape: &Metascape) {
        // Debug time multiplier.
        self.time_multipliers[self.time_multipliers_cursor] = metascape.time_multiplier;
        self.time_multipliers_cursor = (self.time_multipliers_cursor + 1) % self.time_multipliers.len();
    }

    pub fn render(&self, owner: &Node2D, rect: Rect2, font: TRef<Font>) {
        let (mut min, mut max) = self
            .time_multipliers
            .iter()
            .fold((f32::MAX, f32::MIN), |(min, max), x| (min.min(*x), max.max(*x)));
        min *= 0.8;
        max *= 1.25;

        let iter = (self.time_multipliers_cursor..self.time_multipliers.len())
            .into_iter()
            .chain(0..self.time_multipliers_cursor)
            .zip(0..)
            .map(|(i, num)| {
                Vector2::new(
                    (num as f32 / self.time_multipliers.len() as f32) * rect.size.x + rect.position.x,
                    ((self.time_multipliers[i] - min) / max) * rect.size.y + rect.position.y + rect.size.y,
                )
            });
        owner.draw_polyline(
            PoolArray::from_iter(iter),
            Color {
                r: 0.95,
                g: 0.95,
                b: 1.0,
                a: 0.5,
            },
            2.0,
            true,
        );

        owner.draw_string(
            font,
            rect.position,
            format!("{:.2}", max),
            Color {
                r: 0.95,
                g: 0.95,
                b: 1.0,
                a: 1.0,
            },
            -1,
        );
        owner.draw_string(
            font,
            Vector2::new(rect.position.x, rect.position.y + rect.size.y),
            format!("{:.2}", min),
            Color {
                r: 0.95,
                g: 0.95,
                b: 1.0,
                a: 1.0,
            },
            -1,
        );
    }
}
