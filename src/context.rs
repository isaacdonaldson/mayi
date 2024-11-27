// context.rs

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Context {
    connections: HashMap<String, Arc<dyn Any + Send + Sync>>,
    data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            data: HashMap::new(),
        }
    }

    pub fn add_connection<T: Any + Send + Sync>(&mut self, name: &str, connection: T) -> &mut Self {
        self.connections
            .insert(name.to_string(), Arc::new(connection));
        self
    }

    pub fn get_connection<T: Any + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        self.connections
            .get(name)
            .and_then(|c| c.downcast_ref::<T>().map(Arc::clone))
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, key: &str, value: T) -> &mut Self {
        self.data.insert(key.to_string(), Box::new(value));
        self
    }

    pub fn get<T: Any + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|v| v.downcast_ref::<T>())
    }
}
