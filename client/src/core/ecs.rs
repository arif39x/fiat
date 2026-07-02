use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub type EntityId = u64;

#[derive(Default)]
pub struct EcsWorld {
    next_id: EntityId,
    components: HashMap<TypeId, HashMap<EntityId, Box<dyn Any>>>,
}

impl EcsWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add<T: 'static>(&mut self, entity: EntityId, component: T) {
        let type_id = TypeId::of::<T>();
        let entries = self
            .components
            .entry(type_id)
            .or_insert_with(HashMap::new);
        entries.insert(entity, Box::new(component));
    }

    pub fn get<T: 'static>(&self, entity: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|entries| entries.get(&entity))
            .and_then(|any| any.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self, entity: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|entries| entries.get_mut(&entity))
            .and_then(|any| any.downcast_mut::<T>())
    }

    pub fn remove<T: 'static>(&mut self, entity: EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|entries| entries.remove(&entity))
            .is_some()
    }

    pub fn query<T: 'static>(&self) -> Vec<(EntityId, &T)> {
        let type_id = TypeId::of::<T>();
        let mut results = Vec::new();
        if let Some(entries) = self.components.get(&type_id) {
            for (&id, any) in entries {
                if let Some(comp) = any.downcast_ref::<T>() {
                    results.push((id, comp));
                }
            }
        }
        results
    }

    pub fn query_mut<T: 'static>(&mut self) -> Vec<(EntityId, &mut T)> {
        let type_id = TypeId::of::<T>();
        let mut results = Vec::new();
        if let Some(entries) = self.components.get_mut(&type_id) {
            for (&id, any) in entries.iter_mut() {
                if let Some(comp) = any.downcast_mut::<T>() {
                    results.push((id, comp));
                }
            }
        }
        results
    }
}

pub trait Component: 'static {}

pub struct Query<'a, T: 'static> {
    world: &'a EcsWorld,
    _marker: PhantomData<T>,
}

impl<'a, T: 'static> Query<'a, T> {
    pub fn new(world: &'a EcsWorld) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> Vec<(EntityId, &T)> {
        self.world.query::<T>()
    }
}
