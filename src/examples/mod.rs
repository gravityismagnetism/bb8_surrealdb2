use crate::{ConnectionManager, DatabaseSettings};
use bb8::Pool;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use surrealdb::sql::Thing;
use tokio::sync::OnceCell;

mod contactdb;
mod rest;

/// The response from creating a new DB entry is the ID of the new record.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record {
    #[allow(dead_code)]
    pub id: Thing,
}

impl Record {
    pub fn to_raw_id(&self) -> String {
        self.id.id.to_string()
    }
}

/// A sample contact record.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Contact {
    /// Using an Option so that the DB can assign the unique ID when inserting a new record.
    pub id: Option<Thing>,
    pub first: String,
    pub last: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

impl Display for Contact {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Contact {{ id: {:?}, first: {}, last: {}, phone: {:?}, email: {:?} }}",
            self.id, self.first, self.last, self.phone, self.email
        )
    }
}

// Hold a single reference to the pool for all test cases to run in parallel.
static POOL: OnceCell<Arc<Pool<ConnectionManager>>> = OnceCell::const_new();

// Initialize the pool with a memory database.
async fn init_pool() -> Result<Arc<Pool<ConnectionManager>>, Box<dyn Error>> {
    let settings = DatabaseSettings::new_memory_db("test".to_string(), "test".to_string());
    let manager = ConnectionManager::new(settings);
    // Memory databases are single-threaded, so limit the pool size to 1.
    let pool = Pool::builder().max_size(1).build(manager).await?;
    Ok(Arc::new(pool))
}

/// Create a pool if it doesn't already exist and return a reference.
pub async fn get_pool() -> Result<Arc<Pool<ConnectionManager>>, Box<dyn Error>> {
    POOL.get_or_try_init(init_pool)
        .await
        .map(|pool| pool.clone())
}
