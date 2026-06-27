use crate::networking::new::client_id::ClientId;
use arc_swap::ArcSwap;
use hecs::Entity;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::time::Instant;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct UserId(pub u64);

#[derive(Clone)]
pub struct UserData {
    pub client_id: Option<ClientId>,
    pub entity: Option<Entity>,

    pub name: String,
    pub joined_at: Instant,
}

static NEXT_USER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
struct UserRegistry {
    users: HashMap<UserId, UserData>,

    client_to_user_id: HashMap<ClientId, UserId>,
}

impl UserRegistry {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            client_to_user_id: HashMap::new(),
        }
    }

    fn add(&mut self, id: UserId, data: UserData) {
        self.users.insert(id, data);
    }

    fn remove(&mut self, id: &UserId) {
        self.users.remove(id);
    }

    fn get(&self, id: &UserId) -> Option<UserData> {
        self.users.get(id).cloned()
    }

    fn get_all(&self) -> Vec<UserData> {
        self.users.values().cloned().collect()
    }
}

#[derive(Clone)]
pub struct UserRegistryHandle {
    registry: Arc<ArcSwap<UserRegistry>>,
}

impl UserRegistryHandle {
    fn new(registry: UserRegistry) -> Self {
        return Self {
            registry: Arc::new(ArcSwap::from_pointee(registry)),
        };
    }

    fn add(&self, id: UserId, data: UserData) {
        self.registry.rcu(move |r| {
            let mut new = (**r).clone();
            new.add(id.clone(), data.clone());
            new
        });
    }

    pub fn create_user(&self, name: String, client_id: ClientId) {
        let data = UserData {
            name,
            joined_at: Instant::now(),
            client_id: Some(client_id),
            entity: None,
        };

        let id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

        self.add(UserId(id), data);
    }

    pub fn remove(&self, id: &UserId) {
        self.registry.rcu(|r| {
            let mut new = (**r).clone();
            new.remove(id);
            new
        });
    }

    pub fn get(&self, id: &UserId) -> Option<UserData> {
        self.registry.load().get(id)
    }

    pub fn get_all(&self) -> Vec<UserData> {
        self.registry.load().get_all()
    }
}

pub fn create_user_registry() -> UserRegistryHandle {
    let registry = UserRegistry::new();

    UserRegistryHandle::new(registry)
}
