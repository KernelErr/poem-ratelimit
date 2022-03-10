use poem::{get, handler, listener::TcpListener, EndpointExt, Route, Server};
use poem_ratelimit::RateLimiter;

#[handler]
fn hello() -> String {
    "Hello".to_string()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let rate_limiter = RateLimiter::new(client, "./rate_limit.yaml").unwrap();
    let app = Route::new()
        .at("/", get(hello))
        .at("/hello", get(hello))
        .with(rate_limiter);
    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
