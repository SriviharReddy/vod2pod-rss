use tokio::time::{sleep, Duration};
use configs::{conf, Conf, ConfName};
use eyre::eyre; // You may need to add `use eyre::eyre;` if not already present

/// Establishes a connection to Redis, retrying several times on failure.
pub async fn get_redis_client() -> Result<redis::aio::MultiplexedConnection, eyre::Error> {
    const MAX_RETRIES: u32 = 5;
    const RETRY_DELAY: Duration = Duration::from_secs(5);

    // These lines will panic if the config is not set.
    // Ensure REDIS_ADDRESS and REDIS_PORT are in your environment variables.
    let redis_address = conf().get(ConfName::RedisAddress).expect("REDIS_ADDRESS must be set");
    let redis_port = conf().get(ConfName::RedisPort).expect("REDIS_PORT must be set");
    let connection_string = format!("redis://{}:{}/", redis_address, redis_port);

    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        println!("[Redis] Connecting... (Attempt {}/{})", attempt, MAX_RETRIES);

        match redis::Client::open(connection_string.clone()) {
            Ok(client) => match client.get_multiplexed_tokio_connection().await {
                Ok(connection) => {
                    println!("[Redis] Connection successful!");
                    return Ok(connection);
                }
                Err(e) => {
                    let error = eyre::Error::new(e).wrap_err("Failed to get multiplexed connection");
                    eprintln!("[Redis] Error: {:?}", error);
                    last_error = Some(error);
                }
            },
            Err(e) => {
                let error = eyre::Error::new(e).wrap_err("Failed to create Redis client");
                eprintln!("[Redis] Error: {:?}", error);
                last_error = Some(error);
            }
        }

        if attempt < MAX_RETRIES {
            println!("[Redis] Retrying in {:?}...", RETRY_DELAY);
            sleep(RETRY_DELAY).await;
        }
    }

    // If the loop finishes, all retries have failed. Return the last error.
    Err(last_error.unwrap_or_else(|| eyre!("Could not connect to Redis after {} attempts.", MAX_RETRIES)))
}
