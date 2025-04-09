use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use zero::Deref0;

#[derive(Default, Deref0)]
pub struct AnyMap(HashMap<TypeId, Box<dyn Any + Send + Sync>>);

impl AnyMap {
    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) -> Option<T> {
        self.0
            .insert(
                TypeId::of::<T>(),
                Box::new(value) as Box<dyn Any + Send + Sync>,
            )
            .and_then(|b| b.downcast().ok().map(|b| *b))
    }

    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .and_then(|arc| arc.downcast_ref())
    }

    pub fn get_or_insert_with<T, F>(&mut self, f: F) -> &T
    where
        T: Any + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        if !self.0.contains_key(&TypeId::of::<T>()) {
            self.insert(f());
        }
        self.get::<T>().expect("just inserted")
    }
}

#[cfg(test)]
mod test {
    use crate::collections::AnyMap;

    #[test]
    pub fn test_sth() {
        let mut map = AnyMap::default();
        map.insert(12);
        map.insert("hello".to_string());
        println!("{:?} {:?}", map.get::<i32>(), map.get::<String>());
    }
}
