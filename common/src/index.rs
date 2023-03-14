use std::hash::Hash;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::utils::HashMap;

/// An index providing a mapping between components and entities.
/// Values may be stale between frame updates!
#[derive(Resource)]
pub struct Index<T> {
    entity_index: HashMap<T, Entity>,
    // Needed to be able to remove dropped entities from the index
    component_index: HashMap<Entity, T>
}

impl<T: Copy + Eq + Hash> Index<T> {
    pub fn new() -> Self {
        Self { entity_index: HashMap::new(), component_index: HashMap::new() }
    }

    pub fn entity(&self, value: &T) -> Option<Entity> {
        self.entity_index.get(value).copied()
    }

    pub fn insert(&mut self, value: T, entity: Entity) {
        self.entity_index.insert(value, entity);
        self.component_index.insert(entity, value);
    }

    pub fn remove(&mut self, entity: Entity) {
        if let Some(value) = self.component_index.remove(&entity) {
            self.entity_index.remove(&value);
        }
    }
}

fn update_index<T: Copy + Eq + Hash + Send + Sync + Component>(
    mut index: ResMut<Index<T>>,
    added_query: Query<(Entity, &T), Added<T>>,
    mut removed_query: RemovedComponents<T>
) {
    for (entity, value) in added_query.iter() {
        index.insert(*value, entity);
    }

    for entity in removed_query.iter() {
        index.remove(entity);
    }
}

pub struct IndexPlugin<T: Send + Sync + Clone + Copy + PartialEq + Eq + Hash + Component + 'static>(PhantomData<T>);

impl<T: Send + Sync + Clone + Copy + PartialEq + Eq + Hash + Component + 'static> IndexPlugin<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T: Send + Sync + Clone + Copy + PartialEq + Eq + Hash + Component + 'static> Plugin for IndexPlugin<T> {
    fn build(&self, app: &mut App) {
        app.insert_resource(Index::<T>::new())
            .add_system(update_index::<T>.in_base_set(CoreSet::PostUpdate));
    }
}