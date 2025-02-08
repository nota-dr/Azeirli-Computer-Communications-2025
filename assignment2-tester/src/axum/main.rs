mod routes;
mod middlewares;
mod utils;

use axum::extract::Request;
use axum::routing::get;
use axum::Router;
use axum::middleware;
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use std::sync::{Arc, Mutex};
use tower::Service;

struct GlobalState {
    secret: Vec<u8>,
    timestemp: u64,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(GlobalState {
        secret: Vec::new(),
        timestemp: 0,
    }));

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/resources", get(routes::resource_by_query_handler))
        .route("/resources/{file}", get(routes::resource_by_path_handler))
        .route("/recursive/{n}", get(routes::recursive_redirect_handler))
        .with_state(state.clone())
        .route("/secret", get(routes::secret_handler))
        .with_state(state.clone())
        .route("/absolute", get(routes::absolute_redirect_handler))
        .fallback(routes::not_found)
        .layer(middleware::from_fn(middlewares::add_common_headers));

    println!("Axum server is listening on port 8080");
    let listener = tokio::net::TcpListener::bind("localhost:8080")
        .await
        .unwrap();

    // Continuously accept new connections.
    loop {
        // In this example we discard the remote address. See `fn serve_with_connect_info` for how
        // to expose that.
        let (socket, _remote_addr) = listener.accept().await.unwrap();

        // We don't need to call `poll_ready` because `Router` is always ready.
        let tower_service = app.clone();

        // Spawn a task to handle the connection. That way we can handle multiple connections
        // concurrently.
        tokio::spawn(async move {
            // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
            // `TokioIo` converts between them.
            let socket = TokioIo::new(socket);

            // Hyper also has its own `Service` trait and doesn't use tower. We can use
            // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
            // `tower::Service::call`.
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
                // tower's `Service` requires `&mut self`.
                //
                // We don't need to call `poll_ready` since `Router` is always ready.
                tower_service.clone().call(request)
            });

            // `server::conn::auto::Builder` supports both http1 and http2.
            //
            // `TokioExecutor` tells hyper to use `tokio::spawn` to spawn tasks.
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                // `serve_connection_with_upgrades` is required for websockets. If you don't need
                // that you can use `serve_connection` instead.
                .title_case_headers(true)
                .serve_connection(socket, hyper_service)
                .with_upgrades()
                .await
            {
                eprintln!("failed to serve connection: {err:#}");
            }
        });
    }
}
