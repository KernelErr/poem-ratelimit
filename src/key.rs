use poem::web::RemoteAddr;

pub fn global() -> String {
    "ratelimit_global".to_string()
}

pub fn ip(addr: &RemoteAddr) -> String {
    format!("ratelimit_ip_{}", addr)
}

pub fn route(uri: &str) -> String {
    format!("ratelimit_route_{}", uri)
}
