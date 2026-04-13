mod access_points;
mod devices;
mod groups;
mod pfsense;

use std::sync::Arc;

use rspc::{BuiltRouter, Rspc};
use tokio_rusqlite::Connection;

pub(crate) const R: Rspc<Ctx> = Rspc::new();

#[derive(Clone)]
pub struct Ctx {
    pub db: Arc<Connection>,
}

pub fn build_router() -> Arc<BuiltRouter<Ctx>> {
    R.router()
        .merge("devices", devices::router())
        .merge("groups", groups::router())
        .merge("accessPoints", access_points::router())
        .merge("pfsense", pfsense::router())
        .build()
        .unwrap()
        .arced()
}
