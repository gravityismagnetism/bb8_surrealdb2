extern crate bb8;
extern crate surrealdb;

use crate::errors::{ConnectionError, DatabaseConnectionErrors};
use async_trait::async_trait;
use bb8::ManageConnection;
use surrealdb::engine::any;
use surrealdb::engine::any::Any;
#[allow(unused_imports)]
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub mod errors;

#[cfg(feature = "examples")]
pub mod examples;

/// Database types supported by the SurrealDB engine using the 'any::connect' method.
/// See SurrealDB Rust API engine::any::connect for more details.
/// Note that while they are included in this enum and the features for completeness,
/// most of them are not yet implemented / verified within this library.
#[derive(Debug)]
pub enum DatabaseType {
    // mem://
    #[cfg(feature = "kv-memory")]
    Memory,
    // rocksdb://path/to/database-folder
    #[cfg(feature = "kv-rocksdb")]
    File,
    // surrealkv://path/to/database-folder
    #[cfg(feature = "kv-surreal")]
    KeyValue,
    // ws://localhost:8000
    #[cfg(feature = "kv-websocket")]
    WebSocket,
    // wss://cloud.surrealdb.com
    #[cfg(feature = "kv-websocket")]
    WebSocketSecure,
    // http://localhost:8000
    #[cfg(feature = "http")]
    Http,
    // https://cloud.surrealdb.com
    #[cfg(feature = "http")]
    Https,
    // Indxdb://DatabaseName
    #[cfg(feature = "kv-indxdb")]
    Indxdb,
    // tikv://localhost:2379
    #[cfg(feature = "kv-tikv")]
    TiKV,
    // fdb://path/to/fdb.cluster
    #[cfg(feature = "kv-fdb")]
    FoundationDB,
}

#[derive(Debug)]
pub struct DatabaseSettings {
    pub db_type: DatabaseType,
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub path: String,
    pub namespace: String,
    pub database_name: String,
}

impl DatabaseSettings {
    #[cfg(feature = "kv-memory")]
    pub fn new_memory_db(namespace: String, database_name: String) -> Self {
        DatabaseSettings {
            db_type: DatabaseType::Memory,
            username: "".to_string(),
            password: "".to_string(),
            port: 0,
            host: "".to_string(),
            path: "".to_string(),
            namespace,
            database_name,
        }
    }

    #[cfg(feature = "kv-rocksdb")]
    pub fn new_file_db(path: String, namespace: String, database_name: String) -> Self {
        DatabaseSettings {
            db_type: DatabaseType::File,
            username: "".to_string(),
            password: "".to_string(),
            port: 0,
            host: "".to_string(),
            path,
            namespace,
            database_name,
        }
    }
}

pub struct ConnectionManager {
    settings: DatabaseSettings,
}

impl ConnectionManager {
    pub fn new(settings: DatabaseSettings) -> Self {
        ConnectionManager { settings }
    }
}

#[async_trait]
impl ManageConnection for ConnectionManager {
    /// The connection type this manager deals with.
    type Connection = Surreal<Any>;

    /// The error type returned by `Connection`s.
    type Error = ConnectionError;

    /// Connects to a local, remote or embedded database
    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        if self.settings.namespace.is_empty() {
            return Err(ConnectionError {
                error: DatabaseConnectionErrors::InvalidNamespace,
            });
        }
        if self.settings.database_name.is_empty() {
            return Err(ConnectionError {
                error: DatabaseConnectionErrors::InvalidDatabaseName,
            });
        }
        match self.settings.db_type {
            #[cfg(feature = "kv-memory")]
            DatabaseType::Memory => {
                let conn: Surreal<Any> = any::connect("mem://").await.unwrap();
                conn.use_ns(self.settings.namespace.as_str())
                    .use_db(self.settings.database_name.as_str())
                    .await
                    .unwrap();
                Ok(conn)
            }
            #[cfg(feature = "kv-rocksdb")]
            DatabaseType::File => {
                let conn_str = format!("rocksdb://{}", self.settings.path);
                let conn: Surreal<Any> = any::connect(conn_str).await.unwrap();
                conn.use_ns(self.settings.namespace.as_str())
                    .use_db(self.settings.database_name.as_str())
                    .await
                    .unwrap();
                Ok(conn)
            }
            #[cfg(feature = "kv-websocket")]
            DatabaseType::WebSocket => {
                if self.settings.host.is_empty() {
                    return Err(ConnectionError {
                        error: DatabaseConnectionErrors::InvalidHost,
                    });
                }
                if self.settings.port < 1025 {
                    return Err(ConnectionError {
                        error: DatabaseConnectionErrors::InvalidPort,
                    });
                }
                if self.settings.username.is_empty() {
                    return Err(ConnectionError {
                        error: DatabaseConnectionErrors::InvalidUsername,
                    });
                }
                if self.settings.password.is_empty() {
                    return Err(ConnectionError {
                        error: DatabaseConnectionErrors::InvalidPassword,
                    });
                }
                let host = format!("ws://{}:{}", self.settings.host, self.settings.port);
                let conn: Surreal<Any> = any::connect(host).await.unwrap();
                conn.signin(Root {
                    username: self.settings.username.as_str(),
                    password: self.settings.password.as_str(),
                })
                .await
                .unwrap();
                Ok(conn)
            }
            // Reason: Some feature variants are not yet implemented, and this helps to provide
            // an error when features are enabled but variants are not yet supported.
            #[allow(unreachable_patterns)]
            _ => Err(ConnectionError {
                error: DatabaseConnectionErrors::NotYetImplemented,
            }),
        }
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        let health = conn.health().await.is_ok();
        if !health {
            Err(ConnectionError {
                error: DatabaseConnectionErrors::HealthCheckFailed,
            })
        } else {
            Ok(())
        }
    }

    fn has_broken(&self, _: &mut Surreal<Any>) -> bool {
        // TODO: Implement a proper check for non-memory cases
        false
    }
}
