use poem::{get, handler, listener::TcpListener, EndpointExt, Route, Server};
use poem_ratelimit::RateLimiter;
use redis::aio::ConnectionManager;

#[handler]
fn hello() -> String {
    "Hello".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let config = {
        let data = std::fs::read_to_string("./rate_limit.yaml")?;
        serde_yaml::from_str(&data)?
    };
    let rate_limiter = RateLimiter::new(ConnectionManager::new(client).await?, config);

    let app = Route::new()
        .at("/", get(hello))
        .at("/hello", get(hello))
        .with(rate_limiter);
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await?;
    Ok(())
}
