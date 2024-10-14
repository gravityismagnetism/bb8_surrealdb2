# Asynchronous Database Pool Connection Library
This library provides the connection logic to support using SurrealDB v2+ with the asynchronous
'[bb8](https://crates.io/crates/bb8)' connection pool library. It supports feature flags
to enable easier cross-platform development and testing. By default the library will use the
memory storage engine, but it can be configured to use many other SurrealDB storage engine types.

## Usage
To use this library, add the following to your `Cargo.toml` file:
```toml
[dependencies]
async-trait = "0.1"
bb8 = "0.8.5"
bb8_surrealdb2 = "0.3"
serde = { version = "1.0.210", features = ["derive"] }
surreal_db = "2.0"
tokio = { version = "1.40.0", features = [
    "macros",
    "rt-multi-thread",
    "test-util",
] }
```

Within the code, you can use the library as follows:
```rust
// Hold a single reference to the pool for all test cases to run in parallel.
static POOL: OnceCell<Arc<Pool<ConnectionManager>>> = OnceCell::const_new();

// Initialize the pool with a memory database.
async fn init_pool() -> Result<Arc<Pool<ConnectionManager>>, Box<dyn Error>> {
    // Use whichever settings you need for your database.
    let settings = DatabaseSettings::new_memory_db("test".to_string(), "test".to_string());
    let manager = ConnectionManager::new(settings);
    // Set the maximum pool size to whatever you need.
    // Memory databases are single-threaded, so limit the pool size to 1 in this case.
    let pool = Pool::builder().max_size(1).build(manager).await?;
    Ok(Arc::new(pool))
}

/// Create a pool if it doesn't already exist and return a reference.
pub async fn get_pool() -> Result<Arc<Pool<ConnectionManager>>, Box<dyn Error>> {
    POOL.get_or_try_init(init_pool)
        .await
        .map(|pool| pool.clone())
}

// Everywhere you need to use the pool, you can get a connection reference to it like this:
let pool = get_pool().await?;
let connection = pool.get().await?;

// or even better, use a match case to handle potential errors:
match get_pool().await {
    Ok(pool) => match pool.get().await {
        Ok(mut conn) => {
            // Do something with the connection.
        }
        Err(e) => {
            // Handle the error.
        }
    }
    Err(e) => {
        // Handle the error.
    }
}
```


## Example
There are two examples provided in the `examples` directory. The first example demonstrates how to use the library
with a local SurrealDB v2 instance. The second example demonstrates how to use the library with web base applications.

to run these examples, use the following command:
```shell
cargo test -- --nocapture
```

## License
Licensed under the MIT license ([LICENSE](LICENSE.md)).

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work
by you shall be licensed as above, without any additional terms or conditions.
