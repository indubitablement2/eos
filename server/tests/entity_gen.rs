use bevy_ecs::prelude::*;

#[test]
fn test_entity_id() {
    let mut w = World::new();
    let id = w.spawn().id();

    assert!(w.get_entity(Entity::from_raw(id.id())).is_some());

    w.despawn(id);

    assert!(w.get_entity(Entity::from_raw(id.id())).is_none());

    let id2 = w.spawn().id();
    println!("{:?}, {:?}", id, id2);

    assert_eq!(id.id(), id2.id());
    assert!(w.get_entity(id).is_none());
    assert!(w.get_entity(id2).is_some());
}
