#![warn(missing_docs)]

//! Rate limit middleware for poem web framework
//! 
//! This middleware is used to limit the number of requests per second.
//! Redis is used to store data.
//! 
//! For detailed information, please check [examples](https://github.com/devsday/poem-ratelimit/tree/main/examples) and our [website](https://devs.day/poem-ratelimit).

mod config;
mod error;
mod key;
mod middleware;

pub use config::{Config, ConfigRecord};
pub use error::RateLimitError;
pub use middleware::{RateLimiter, RateLimiterImpl};
