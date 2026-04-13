mod db;
mod models;
mod router;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use rspc::ExportConfig;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Arc::new(db::open("network_devices.db").await?);

    let rspc_router = router::build_router();

    // Export TypeScript bindings during development (or always for simplicity)
    if let Err(e) = rspc_router.export_ts(ExportConfig::new("../frontend/src/bindings.ts")) {
        eprintln!("Warning: failed to export TypeScript bindings: {}", e);
    }

    let app = Router::new()
        .nest(
            "/rspc",
            rspc_router
                .endpoint(move || router::Ctx { db: db.clone() })
                .axum(),
        )
        .layer(CorsLayer::very_permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Backend listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

