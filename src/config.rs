use std::collections::HashMap;

use poem::http::Uri;
use serde::de::Error;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Config of rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Limit for all requests
    pub global: Option<ConfigRecord>,
    /// Limit for a g client IP
    pub ip: Option<ConfigRecord>,
    /// Limit for a specific route
    #[serde(
        deserialize_with = "deserialize_route",
        serialize_with = "serialize_route",
        default
    )]
    pub route: Option<HashMap<Uri, ConfigRecord>>,
}

fn serialize_route<S>(
    route: &Option<HashMap<Uri, ConfigRecord>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match route {
        Some(route) => {
            let mut s = serializer.serialize_map(Some(route.len()))?;
            for (k, v) in route {
                s.serialize_entry(&k.to_string(), v)?;
            }
            s.end()
        }
        None => serializer.serialize_none(),
    }
}

fn deserialize_route<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<Uri, ConfigRecord>>, D::Error>
where
    D: Deserializer<'de>,
{
    let route: Option<HashMap<String, ConfigRecord>> = Deserialize::deserialize(deserializer)?;
    match route {
        Some(route) => Ok(Some(route.into_iter().try_fold(
            HashMap::new(),
            |mut acc, (k, v)| {
                let uri = k
                    .parse::<Uri>()
                    .map_err(|err| D::Error::custom(err.to_string()))?;
                acc.insert(uri, v);
                Ok(acc)
            },
        )?)),
        None => Ok(None),
    }
}

/// Rate limit rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRecord {
    /// How many requests can be made in a time window
    pub max_requests: usize,
    /// Time window in seconds
    pub time_window: usize,
}
