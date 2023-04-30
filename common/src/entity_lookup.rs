use bevy::{prelude::*, ecs::query::ReadOnlyWorldQuery};

pub fn lookup<T: Component + PartialEq, F: ReadOnlyWorldQuery>(
    query: &Query<(Entity, &T), F>,
    val: &T
) -> Option<Entity> {
    for (entity, component) in query.iter() {
        if component == val {
            return Some(entity);
        }
    }

    None
}

pub fn lookup_exclusive<T: Component + PartialEq, F: ReadOnlyWorldQuery>(
    world: &mut World,
    query: &mut QueryState<(Entity, &T), F>,
    val: &T
) -> Option<Entity> {
    for (entity, component) in query.iter(world) {
        if component == val {
            return Some(entity);
        }
    }

    None
}