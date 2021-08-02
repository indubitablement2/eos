use eos_common::{
    const_var::{SECTOR_SIZE, X_SECTOR},
    idx::SectorId,
    location::*,
};
use glam::{vec2, Vec2};

#[test]
fn distance() {
    let from = Location {
        sector_id: SectorId(0),
        local_position: Vec2::ZERO,
    };
    let to = Location {
        sector_id: SectorId(1),
        local_position: Vec2::ZERO,
    };

    assert_eq!(from.euclid_direction(to), vec2(SECTOR_SIZE, 0.0));

    let from = Location {
        sector_id: SectorId(0),
        local_position: Vec2::ZERO,
    };
    let to = Location {
        sector_id: SectorId(1),
        local_position: vec2(1000.0, -1000.0),
    };

    assert_eq!(from.euclid_direction(to), vec2(SECTOR_SIZE + 1000.0, -1000.0));

    let from = Location {
        sector_id: SectorId(1),
        local_position: vec2(1000.0, -1000.0),
    };
    let to = Location {
        sector_id: SectorId(0),
        local_position: Vec2::ZERO,
    };

    assert_eq!(from.euclid_direction(to), -vec2(SECTOR_SIZE + 1000.0, -1000.0));

    let from = Location {
        sector_id: SectorId(0),
        local_position: Vec2::ZERO,
    };
    let to = Location {
        sector_id: SectorId(X_SECTOR as u16 + 1),
        local_position: Vec2::ZERO,
    };

    assert_eq!(from.euclid_direction(to), vec2(SECTOR_SIZE, SECTOR_SIZE));
}
