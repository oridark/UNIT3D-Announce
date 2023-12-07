pub mod blacklisted_agent;
pub mod blacklisted_port;
pub mod freeleech_token;
pub mod group;
pub mod peer;
pub mod personal_freeleech;
pub mod torrent;
pub mod user;

use parking_lot::RwLock;
pub use peer::Peer;
pub use torrent::Torrent;
pub use user::User;

use sqlx::MySqlPool;

use anyhow::{Context, Result};

use crate::config;
use crate::scheduler::{history_update, peer_update, torrent_update, user_update};
use crate::stats::Stats;

use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::{env, sync::Arc, time::Duration};

pub struct Tracker {
    pub agent_blacklist: blacklisted_agent::Set,
    pub config: config::Config,
    pub freeleech_tokens: freeleech_token::Set,
    pub groups: group::Map,
    pub history_updates: RwLock<history_update::Queue>,
    pub infohash2id: torrent::infohash2id::Map,
    pub passkey2id: user::passkey2id::Map,
    pub peer_updates: RwLock<peer_update::Queue>,
    pub personal_freeleeches: personal_freeleech::Set,
    pub pool: MySqlPool,
    pub port_blacklist: blacklisted_port::Set,
    pub stats: Stats,
    pub torrents: torrent::Map,
    pub torrent_updates: RwLock<torrent_update::Queue>,
    pub users: user::Map,
    pub user_updates: RwLock<user_update::Queue>,
}

impl Tracker {
    /// Creates a database connection pool, and loads all relevant tracker
    /// data into this shared tracker context. This is then passed to all
    /// handlers.
    pub async fn default() -> Result<Arc<Tracker>> {
        println!(".env file: verifying file exists...");
        dotenv().context(".env file not found.")?;
        println!("\x1B[1F\x1B[2KFound .env file");

        println!("Loading from database into memory: config...");
        let config = config::Config::from_env()?;
        println!("\x1B[1F\x1B[2KLoaded config parameters");

        println!("Connecting to database...");
        let pool = connect_to_database().await;
        println!("\x1B[1F\x1B[2KConnected to database");

        println!("Loading from database into memory: blacklisted ports...");
        let port_blacklist = blacklisted_port::Set::default();
        println!(
            "\x1B[1F\x1B[2KLoaded {:?} blacklisted ports",
            port_blacklist.len()
        );

        println!("Loading from database into memory: blacklisted user agents...");
        let agent_blacklist = blacklisted_agent::Set::from_db(&pool).await?;
        println!(
            "\x1B[1F\x1B[2KLoaded {:?} blacklisted agents",
            agent_blacklist.len()
        );

        println!("Loading from database into memory: torrents...");
        let torrents = torrent::Map::from_db(&pool).await?;
        println!("\x1B[1F\x1B[2KLoaded {:?} torrents", torrents.len());

        println!("Loading from database into memory: infohash to torrent id mapping...");
        let infohash2id = torrent::infohash2id::Map::from_db(&pool).await?;
        println!(
            "\x1B[1F\x1B[2KLoaded {:?} torrent infohash to id mappings",
            infohash2id.len()
        );

        println!("Loading from database into memory: users...");
        let users = user::Map::from_db(&pool).await?;
        println!("\x1B[1F\x1B[2KLoaded {:?} users", users.len());

        println!("Loading from database into memory: passkey to user id mapping...");
        let passkey2id = user::passkey2id::Map::from_db(&pool).await?;
        println!(
            "\x1B[1F\x1B[2KLoaded {:?} user passkey to id mappings",
            passkey2id.len()
        );

        println!("Loading from database into memory: freeleech tokens...");
        let freeleech_tokens = freeleech_token::Set::from_db(&pool).await?;
        println!(
            "\x1B[1F\x1B[2KLoaded {:?} freeleech tokens",
            freeleech_tokens.len()
        );

        println!("Loading from database into memory: personal freeleeches...");
        let personal_freeleeches = personal_freeleech::Set::from_db(&pool).await?;
        println!(
            "\x1B[1F\x1B[2KLoaded {:?} personal freeleeches",
            personal_freeleeches.len()
        );

        println!("Loading from database into memory: groups...");
        let groups = group::Map::from_db(&pool).await?;
        println!("\x1B[1F\x1B[2KLoaded {:?} groups", groups.len());

        let stats = Stats::default();

        Ok(Arc::new(Tracker {
            agent_blacklist,
            config,
            freeleech_tokens: freeleech_tokens,
            groups,
            history_updates: RwLock::new(history_update::Queue::new()),
            infohash2id,
            passkey2id,
            peer_updates: RwLock::new(peer_update::Queue::new()),
            personal_freeleeches,
            pool,
            port_blacklist,
            stats,
            torrents,
            torrent_updates: RwLock::new(torrent_update::Queue::new()),
            users,
            user_updates: RwLock::new(user_update::Queue::new()),
        }))
    }
}

/// Uses the values in the .env file to create a connection pool to the database
async fn connect_to_database() -> sqlx::Pool<sqlx::MySql> {
    // Get pool of database connections.
    MySqlPoolOptions::new()
        .min_connections(0)
        .max_connections(10)
        .max_lifetime(Duration::from_secs(30 * 60))
        .idle_timeout(Duration::from_secs(10 * 60))
        .acquire_timeout(Duration::from_secs(30))
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL not found in .env file. Aborting."))
        .await
        .expect("Could not connect to the database using the DATABASE_URL value in .env file. Aborting.")
}
