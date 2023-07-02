# no_empty_query_params

## Sumary

Middleware layer that removes params with empty values.

## Motivation

In [nginx](https://nginx.org/en/) configuration, when rewriting URLs (e.g. to pass to the upstream server), query params can be interpolated using [`$arg_`](https://nginx.org/en/docs/http/ngx_http_core_module.html#var_arg_)

Example:

```
proxy_pass http://upstream/some/route?format=$arg_param;
```

[Axum](https://github.com/tokio-rs/axum) makes it easy to write handlers that automatically deserialize the query parameters:

```rust
#[derive(Deserialize)]
enum Format {
    Json,
    Html,
    Text
}

#[derive(Deserialize)]
struct RouteArgs {
    format: Option<Format>
}
```

Annoyingly, the above example will fail. Since `format=` is always unconditionally passed to the handler via the `nginx` configuratio, the default deserialization will try to deserialize `Format` from `""`.
