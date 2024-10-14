extern crate bb8;
extern crate once_cell;
extern crate tokio;

use bb8::Pool;
use bb8_surrealdb2::{ConnectionManager, DatabaseSettings};
use std::sync::Arc;
use tokio::sync::OnceCell;

static POOL: OnceCell<Arc<Pool<ConnectionManager>>> = OnceCell::const_new();

async fn init_pool() -> Result<Arc<Pool<ConnectionManager>>, Box<dyn std::error::Error>> {
    let settings = DatabaseSettings::new_memory_db("test".to_string(), "test".to_string());
    let manager = ConnectionManager::new(settings);
    let pool = Pool::builder().max_size(3).build(manager).await?;
    Ok(Arc::new(pool))
}

async fn get_pool() -> Result<Arc<Pool<ConnectionManager>>, Box<dyn std::error::Error>> {
    POOL.get_or_try_init(init_pool)
        .await
        .map(|pool| pool.clone())
}

/// Allows running multiple tests in parallel to demonstrate the connection pool use.
/// For more useful examples, see the examples directory.
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_version_simple() {
        // Rely on the Panic to handle failures.
        let pool = get_pool().await.unwrap();
        let connection = pool.get().await.unwrap();
        let version = connection.version().await.unwrap();
        println!("Version={}", version);
        assert_ne!(version.to_string(), "");
    }

    #[tokio::test]
    async fn test_version_via_matches() {
        // Use the match to handle failures that provide more explicit error messages.
        match get_pool().await {
            Ok(pool) => match pool.get().await {
                Ok(connection) => match connection.version().await {
                    Ok(version) => {
                        println!("Version={}", version);
                        assert_ne!(version.to_string(), "");
                    }
                    Err(e) => assert!(false, "Failed to get version: {:?}", e),
                },
                Err(e) => assert!(false, "Failed to get connection: {:?}", e),
            },
            Err(e) => assert!(false, "Failed to initialize pool: {:?}", e),
        }
    }
}
