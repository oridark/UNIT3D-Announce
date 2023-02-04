use std::ops::Deref;

use dashmap::DashMap;
use sqlx::types::chrono::{DateTime, Utc};

pub mod peer_id;
pub use peer_id::PeerId;
pub mod user_agent;
pub use user_agent::UserAgent;

pub struct Map(DashMap<Index, Peer>);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Index {
    pub user_id: u32,
    pub peer_id: PeerId,
}

#[derive(Clone, Copy, Debug)]
pub struct Peer {
    pub ip_address: std::net::IpAddr,
    pub user_id: u32,
    pub port: u16,
    pub is_seeder: bool,
    pub is_active: bool,
    pub updated_at: DateTime<Utc>,
    pub uploaded: u64,
    pub downloaded: u64,
}

impl Map {
    pub fn new() -> Map {
        Map(DashMap::new())
    }
}

impl Deref for Map {
    type Target = DashMap<Index, Peer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Map {
    fn default() -> Self {
        Map::new()
    }
}
