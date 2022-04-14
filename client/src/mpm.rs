use std::ops::Sub;

use rand::Rng;

pub struct MPM {
    pub dom: glam::UVec2,
    grid: Vec<Cell>,
    pub particles: Vec<Particle>,
}
impl MPM {
    const GRAVITY: f32 = -0.05;

    pub fn new(dom: glam::UVec2) -> Self {
        let grid = vec![
            Cell {
                vel: glam::Vec2::ZERO,
                mass: 0.0
            };
            (dom.x * dom.y) as usize
        ];

        // Initialize particles with random values.
        let mut rng = rand::thread_rng();
        let x_min = dom.x as f32 * 0.1;
        let x_max = dom.x as f32 * 0.8;
        let y_min = dom.y as f32 * 0.1;
        let y_max = dom.y as f32 * 0.8;
        let particles = (0..256)
            .map(|_| Particle {
                pos: glam::vec2(rng.gen_range(x_min..x_max), rng.gen_range(y_min..y_max)),
                vel: glam::vec2(rng.gen_range(-0.01f32..0.01), rng.gen_range(-0.01f32..0.01)),
                mass: 1.0,
            })
            .collect();

        Self {
            dom,
            grid,
            particles,
        }
    }

    pub fn update(&mut self) {
        // Reset grid.
        for cell in self.grid.iter_mut() {
            *cell = Cell::default();
        }

        // Transfer data from particles to grid.
        for p in self.particles.iter() {
            let weights = weights_interpolation(p.pos);

            // For all surrounding 9 cells.
            for x in 0..3 {
                for y in 0..3 {
                    let weight = weights[x].x * weights[y].y;

                    let cell_idx = p.pos.sub(1.0).as_uvec2() + glam::uvec2(x as u32, y as u32);
                    let cell_dist = cell_idx.as_vec2() - p.pos + 0.5;
                    // TODO: Momentum matrix.

                    // Get the cell.
                    let cell_id = cell_idx.x + cell_idx.y * self.dom.x;
                    if cell_id >= self.grid.len() as u32 {
                        continue;
                    }
                    let cell = &mut self.grid[cell_id as usize];

                    let mass_contrib = weight * p.mass;

                    // Scatter mass.
                    cell.mass += mass_contrib;

                    // Scatter momemtum.
                    cell.vel += mass_contrib * p.vel;
                }
            }
        }

        // Grid velocity update.
        for cell in self.grid.iter_mut() {
            if cell.mass <= 0.0 {
                continue;
            }

            // Convert momemtum to velocity.
            cell.vel /= cell.mass;

            // Apply gravity.
            cell.vel.y += MPM::GRAVITY;
        }

        // Boundary conditions.
        for i in 0..self.dom.x {
            // Top row.
            let cell = &mut self.grid[i as usize];
            let y = cell.vel.y.min(0.0);
            cell.vel.y = y;

            // Bottom row.
            let base = self.dom.x * (self.dom.y - 1);
            let cell = &mut self.grid[(i + base) as usize];
            let y = cell.vel.y.max(0.0);
            cell.vel.y = y;
        }
        for i in 1..self.dom.y - 1 {
            // Right column.
            let i = i * self.dom.x;
            let cell = &mut self.grid[i as usize];
            let x = cell.vel.x.min(0.0);
            cell.vel.x = x;

            // Left column.
            let cell = &mut self.grid[(i + 1) as usize];
            let x = cell.vel.x.max(0.0);
            cell.vel.x = x;
        }
        // Top left corner.
        if let Some(cell) = self.grid.first_mut() {
            let x = cell.vel.x.max(0.0);
            cell.vel.x = x;
        }
        // Bot right corner.
        if let Some(cell) = self.grid.last_mut() {
            let x = cell.vel.x.min(0.0);
            cell.vel.x = x;
        }

        // Grid to cell.
        for p in self.particles.iter_mut() {
            // Reset particle velocty.
            p.vel = glam::Vec2::ZERO;

            // ?????????
            let mut b = glam::Mat2::ZERO;
            let weights = weights_interpolation(p.pos);
            for x in 0..3 {
                for y in 0..3 {
                    let weight = weights[x].x * weights[y].y;

                    let cell_idx = p.pos.sub(1.0).as_uvec2() + glam::uvec2(x as u32, y as u32);
                    let cell_id = cell_idx.x + cell_idx.y * self.dom.x;

                    let dist = cell_idx.as_vec2() - p.pos + 0.5;
                    if cell_id >= self.grid.len() as u32 {
                        continue;
                    }
                    let weighted_vel = self.grid[cell_id as usize].vel * weight;

                    b += glam::mat2(weighted_vel * dist.x, weighted_vel * dist.y);

                    p.vel += weighted_vel;
                }
            }

            // TODO: Momentum matrix.

            // Advect particles.
            p.pos += p.vel;

            // Safety clamp.
            p.pos.clamp(glam::vec2(0.1, 0.5), self.dom.as_vec2() - 1.1);
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Cell {
    vel: glam::Vec2,
    mass: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub pos: glam::Vec2,
    pub vel: glam::Vec2,
    pub mass: f32,
}

/// Quadratic interpolation weights.
fn weights_interpolation(pos: glam::Vec2) -> [glam::Vec2; 3] {
    let cell_diff = pos - pos.floor() - 0.5;
    [
        (0.5 - cell_diff) * (0.5 - cell_diff) * 0.5,
        cell_diff * cell_diff - 0.75,
        (0.5 + cell_diff) * (0.5 + cell_diff) * 0.5,
    ]
}
