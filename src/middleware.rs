use poem::error::InternalServerError;
use poem::{async_trait, Endpoint, Middleware, Request, Result};
use redis::aio::ConnectionLike;
use redis::Script;

use crate::{key, Config, ConfigRecord, RateLimitError};

/// Rate Limiter Middleware
/// This middleware is used to limit the number of requests per second.
/// A sliding window script is invoked when a request comes in. Redis is used to store data.
pub struct RateLimiter<C> {
    connection: C,
    config: Config,
}

impl<C: ConnectionLike + Clone + Sync + Send> RateLimiter<C> {
    pub fn new(connection: C, config: Config) -> Self {
        RateLimiter { connection, config }
    }
}

impl<C, E> Middleware<E> for RateLimiter<C>
where
    C: ConnectionLike + Clone + Sync + Send,
    E: Endpoint,
{
    type Output = RateLimiterImpl<C, E>;

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
            redis: self.connection.clone(),
            script,
            config: self.config.clone(),
        }
    }
}

pub struct RateLimiterImpl<C, E> {
    endpoint: E,
    redis: C,
    script: Script,
    config: Config,
}

impl<C, E> RateLimiterImpl<C, E>
where
    C: ConnectionLike + Clone + Sync + Send,
    E: Endpoint,
{
    async fn check(&self, key: &str, cfg_record: &ConfigRecord) -> Result<()> {
        let mut connection = self.redis.clone();
        let max_requests = cfg_record.max_requests;
        let time_window = cfg_record.time_window;

        let res = self
            .script
            .arg(key)
            .arg(time_window)
            .arg(max_requests)
            .invoke_async::<_, i32>(&mut connection)
            .await
            .map_err(InternalServerError)?;

        if res == 0 {
            Ok(())
        } else {
            Err(RateLimitError.into())
        }
    }
}

#[async_trait]
impl<C, E> Endpoint for RateLimiterImpl<C, E>
where
    C: ConnectionLike + Clone + Sync + Send,
    E: Endpoint,
{
    type Output = E::Output;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        if let Some(global_config) = &self.config.global {
            let key = key::global();
            self.check(&key, global_config).await?;
        } else if let Some(ip_config) = &self.config.ip {
            let remote_addr = req.remote_addr().clone();
            let key = key::ip(&remote_addr);
            self.check(&key, ip_config).await?;
        } else if let Some(route_config) = &self.config.route {
            let uri = req.uri();
            if let Some(route_config_record) = route_config.get(uri) {
                let key = key::route(uri);
                self.check(&key, route_config_record).await?;
            }
        }

        self.endpoint.call(req).await
    }
}
