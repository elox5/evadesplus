use crate::{
    game::{area::AreaKey, player::PlayerId},
    networking::new::client_id::ClientId,
};
use arc_swap::ArcSwap;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::time::Instant;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct UserId(pub u64);

#[derive(Clone)]
pub struct UserData {
    pub client_id: Option<ClientId>,
    pub player_id: PlayerId,

    pub name: String,
    pub joined_at: Instant,

    victories: Vec<AreaKey>,
}

static NEXT_USER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
struct UserRegistry {
    users: HashMap<UserId, UserData>,

    client_to_user_id_map: HashMap<ClientId, UserId>,
}

impl UserRegistry {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            client_to_user_id_map: HashMap::new(),
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

    fn add_to_client_map(&mut self, id: UserId, client_id: ClientId) {
        self.client_to_user_id_map.insert(client_id, id);
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

    pub fn create_user(&self, name: String, client_id: ClientId, player_id: PlayerId) -> UserId {
        let data = UserData {
            name,
            joined_at: Instant::now(),
            client_id: Some(client_id),
            player_id,
            victories: Vec::new(),
        };

        let id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
        let id = UserId(id);
        let id_clone = id.clone();

        self.registry.rcu(move |r| {
            let mut new = (**r).clone();
            new.add(id.clone(), data.clone());
            new.add_to_client_map(id.clone(), client_id);
            new
        });

        id_clone
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

    pub fn client_to_user_id(&self, client_id: ClientId) -> Option<UserId> {
        self.registry
            .load()
            .client_to_user_id_map
            .get(&client_id)
            .cloned()
    }
}

pub fn create_user_registry() -> UserRegistryHandle {
    let registry = UserRegistry::new();

    UserRegistryHandle::new(registry)
}
