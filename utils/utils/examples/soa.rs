use utils::*;

#[derive(Fields, Columns, Components, Default)]
struct Data {
    name: String,
    id: u64,
    position: (f32, f32),
    velocity: (f32, f32),
}

pub fn main() {
    let mut map: PackedMap<Soa<Data>, Data, u64> = PackedMap::with_capacity(512);

    let idx: Vec<u64> = (0..512)
        .map(|id| {
            map.insert(
                id,
                Data {
                    id,
                    ..Default::default()
                },
            );
            id
        })
        .collect();
    assert_eq!(map.len(), 512);

    let first_id = idx.first().unwrap().to_owned();

    let (position_ptr, velocity_ptr) = query_ptr!(map.container(), Data::position, Data::velocity);

    for i in 0..map.len() {
        let id = map.get_id(i).unwrap();
        let (position, velocity) = unsafe { (&mut *position_ptr.add(i), &*velocity_ptr.add(i)) };

        if id != first_id {
            let first_velocity = query!(map.container(), i, Data::velocity).0;

            position.0 += first_velocity.0;
            position.1 += first_velocity.1;
        } else {
            position.0 += velocity.0;
            position.1 += velocity.1;
        }
    }
}
