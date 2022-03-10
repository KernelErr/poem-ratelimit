use anyhow::Result;
use poem::{
    async_trait, http::StatusCode, Endpoint, IntoResponse, Middleware, Request, Response,
    Result as PoemResult,
};
use redis::{Client, Script};

mod config;
mod key;

/// Rate Limiter Middleware
/// This middleware is used to limit the number of requests per second.
/// A sliding window script is invoked when a request comes in. Redis is used to store data.
pub struct RateLimiter {
    redis: Client,
    config: config::Config,
}

impl RateLimiter {
    pub fn new(redis: Client, config: &str) -> Result<Self> {
        let config = config::Config::from_file(config)?;
        Ok(RateLimiter { redis, config })
    }
}

impl<E: Endpoint> Middleware<E> for RateLimiter {
    type Output = RateLimiterImpl<E>;

    fn transform(&self, endpoint: E) -> Self::Output {
        // Sliding window algorithm
        // Script from https://developer.redis.com/develop/dotnet/aspnetcore/rate-limiting/sliding-window/
        let script = Script::new(
            r"
            local current_time = redis.call('TIME')
            local trim_time = tonumber(current_time[1]) - ARGV[2]
            redis.call('ZREMRANGEBYSCORE', ARGV[1], 0, trim_time)
            local request_count = redis.call('ZCARD', ARGV[1])

            if request_count < tonumber(ARGV[3]) then
                redis.call('ZADD', ARGV[1], current_time[1], current_time[1] .. current_time[2])
                redis.call('EXPIRE', ARGV[1], ARGV[2])
                return 0
            end
            return 1
        ",
        );
        RateLimiterImpl {
            endpoint,
            redis: self.redis.clone(),
            script,
            config: self.config.clone(),
        }
    }
}

pub struct RateLimiterImpl<E> {
    endpoint: E,
    redis: Client,
    script: Script,
    config: config::Config,
}

#[async_trait]
impl<E: Endpoint> Endpoint for RateLimiterImpl<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> PoemResult<Self::Output> {
        let mut con = self.redis.clone();

        if let Some(global_config) = &self.config.global {
            let key = key::global();
            let max_requests = global_config.max_requests;
            let time_window = global_config.time_window;
            let result = self
                .script
                .arg(&key)
                .arg(time_window)
                .arg(max_requests)
                .invoke::<i32>(&mut con);
            match result {
                Ok(0) => {}
                _ => {
                    return Ok(Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .body("Too many requests"))
                }
            }
        };

        if let Some(ip_config) = &self.config.ip {
            let remote_addr = req.remote_addr().clone();
            let key = key::ip(&remote_addr);
            let max_requests = ip_config.max_requests;
            let time_window = ip_config.time_window;
            let result = self
                .script
                .arg(&key)
                .arg(time_window)
                .arg(max_requests)
                .invoke::<i32>(&mut con);
            match result {
                Ok(0) => {}
                _ => {
                    return Ok(Response::builder()
                        .status(StatusCode::TOO_MANY_REQUESTS)
                        .body("Too many requests"))
                }
            }
        };

        if let Some(route_config) = &self.config.route {
            let uri = req.uri().to_string();
            if let Some(route_config_record) = route_config.get(&uri) {
                let key = key::route(&uri);
                let max_requests = route_config_record.max_requests;
                let time_window = route_config_record.time_window;
                let result = self
                    .script
                    .arg(&key)
                    .arg(time_window)
                    .arg(max_requests)
                    .invoke::<i32>(&mut con);
                match result {
                    Ok(0) => {}
                    _ => {
                        return Ok(Response::builder()
                            .status(StatusCode::TOO_MANY_REQUESTS)
                            .body("Too many requests"))
                    }
                }
            }
        };

        let res = self.endpoint.call(req).await;

        match res {
            Ok(resp) => {
                let resp = resp.into_response();
                Ok(resp)
            }
            Err(err) => Err(err),
        }
    }
}
