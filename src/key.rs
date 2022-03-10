use poem::http::Uri;
use poem::web::RemoteAddr;

pub(crate) fn global() -> String {
    "ratelimit_global".to_string()
}

pub(crate) fn ip(addr: &RemoteAddr) -> String {
    format!("ratelimit_ip_{}", addr)
}

pub(crate) fn route(uri: &Uri) -> String {
    format!("ratelimit_route_{}", uri)
}
