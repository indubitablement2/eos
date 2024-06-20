use super::*;

#[derive(Default)]
pub struct Characters {
    pub characters: Vec<CharacterRef>,
    grid: HashMap<IVec2, SmallVec<[CharacterRef; 3]>>,
    ignore: HashSet<CharacterRef>,
}
impl Characters {
    pub fn update_query_structure(&mut self) {
        self.grid.clear();
        for char in self.characters.iter() {
            let Some(char_ref) = char.get() else {
                continue;
            };
            let start = (char_ref.position - char_ref.radius.value())
                .floor()
                .as_ivec2();
            let end = (char_ref.position + char_ref.radius.value())
                .ceil()
                .as_ivec2();
            for y in start.y..end.y {
                for x in start.x..end.x {
                    self.grid
                        .entry(IVec2::new(x, y))
                        .or_default()
                        .push(char.clone());
                }
            }
        }
    }

    /// Add a character to be ignored for the next query.
    /// This is cleared after the query is done.
    pub fn add_ignore(&mut self, char: CharacterRef) {
        self.ignore.insert(char);
    }

    /// Return all characters in the cells that intersect the given rectangle.
    pub fn query_broad(&mut self, start: Vec2, end: Vec2, f: impl Fn(&CharacterRef) -> bool) {
        let start = (start / CELL_SIZE).floor().as_ivec2();
        let end = (end / CELL_SIZE).ceil().as_ivec2();

        'outer: for y in start.y..end.y {
            for x in start.x..end.x {
                let Some(characters) = self.grid.get(&IVec2::new(x, y)) else {
                    continue;
                };
                for char_ref in characters {
                    if !self.ignore.insert(char_ref.clone()) {
                        continue;
                    }
                    if !f(char_ref) {
                        break 'outer;
                    }
                }
            }
        }

        self.ignore.clear();
    }

    /// Return all characters that intersect the given circle.
    pub fn query_circle(&mut self, position: Vec2, radius: f32, f: impl Fn(&CharacterRef) -> bool) {
        let start = position - radius;
        let end = position + radius;
        self.query_broad(start, end, |char_ref| {
            if let Some(char) = char_ref.get() {
                if (char.position - position).length_squared()
                    < (char.radius.value() + radius).powi(2)
                {
                    f(char_ref)
                } else {
                    true
                }
            } else {
                true
            }
        });
    }
}
