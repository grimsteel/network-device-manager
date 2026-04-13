mod db;
mod models;
mod router;

use std::net::SocketAddr;

use axum::Router;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = db::open("network_devices.db").await?;

    let procedures = router::build();

    let app = Router::new()
        .nest("/rspc", rspc_axum::endpoint(procedures, move || db.clone()))
        .layer(CorsLayer::very_permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Backend listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
