use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
    time::SystemTime,
};

use serde::Serialize;
use tokio::sync::RwLock;

#[derive(Clone, Serialize)]
pub struct Server {
    pub name: String,
    pub hardware: String,
    pub antenna: String,
    pub bandwidth: f64,
    pub users: i32,
    pub remarks: String,
    pub description: String,
    pub base_frequency: f64,
    pub url: String,
    pub last_update: SystemTime,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct ServerLastUpdate {
    last_update: SystemTime,
    id: String,
}

pub type SharedServerList = Arc<RwLock<ServerList>>;

pub struct ServerList {
    servers: HashMap<String, Server>,
    last_updates: BTreeSet<ServerLastUpdate>,
    last_check: SystemTime,
}

impl Default for ServerList {
    fn default() -> Self {
        ServerList {
            servers: HashMap::new(),
            last_updates: BTreeSet::new(),
            last_check: SystemTime::now(),
        }
    }
}

impl ServerList {
    pub fn add_server(&mut self, id: String, server: Server) {
        self.remove_old_servers();
        // Bump it up the queue
        match self.servers.get(&id).map(|s| s.last_update.clone()) {
            Some(previous_update) => {
                self.last_updates.remove(&ServerLastUpdate {
                    last_update: previous_update,
                    id: id.clone(),
                });
            }
            _ => {}
        }
        let last_update = server.last_update.clone();
        self.servers.insert(id.clone(), server);
        self.last_updates
            .insert(ServerLastUpdate { last_update, id });
    }

    pub fn get_all_servers(&self) -> Vec<Server> {
        self.servers.values().cloned().collect()
    }

    pub fn remove_old_servers(&mut self) {
        let now = SystemTime::now();
        // Check once every minute
        if now.duration_since(self.last_check).unwrap().as_secs() < 60 {
            return;
        }
        // Remove servers that haven't been updated in 5 minutes
        while self.last_updates.len() > 0 {
            let oldest = self.last_updates.iter().next().unwrap().clone();
            if now.duration_since(oldest.last_update).unwrap().as_secs() < 60 * 5 {
                break;
            }
            self.servers.remove(&oldest.id);
            self.last_updates.remove(&oldest);
        }
    }
}
