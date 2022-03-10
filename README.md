# Rate limit middleware for Poem framework

## Usage

Check [examples](./examples), `poem-ratelimit` is available on [crates.io](https://crates.io/crates/poem-ratelimit).

A `yaml` configuration file is used to set limit for global service, per IP and route:

```yaml
global:
  # Global limit for all connections
  max_requests: 20
  # Seconds to refresh limit, we allow 20req/30s here
  time_window: 30
ip:
  # QPS limit for a single client IP
  max_requests: 10
  time_window: 30
route:
  /:
    # QPS limit for a single route
    max_requests: 5
    time_window: 30
```

## How it works

For every requests, we use sliding window algorithm to check if the request is processable. Sorted lists with key like IP, route is stored in Redis. You can check the `lua` script in `lib.rs`.

## License

`poem-ratelimit` is licensed under Apache-2.0.