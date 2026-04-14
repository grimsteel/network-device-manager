mod access_points;
mod devices;
mod groups;
mod pfsense;

use std::{borrow::Cow, collections::HashMap, sync::Arc};

use rspc_procedure::{Procedure, ProcedureError, Procedures, ResolverError, State};
use tokio_rusqlite::Connection;

pub type Ctx = Connection;

/// Convert any display-able error into a resolver-level ProcedureError.
pub(crate) fn internal_err(msg: impl std::fmt::Display) -> ProcedureError {
    ProcedureError::Resolver(ResolverError::new(
        serde_json::json!({"code": 500, "message": msg.to_string()}),
        None::<std::io::Error>,
    ))
}

pub fn build() -> Procedures<Ctx> {
    let mut map: HashMap<Cow<'static, str>, Procedure<Ctx>> = HashMap::new();

    devices::register(&mut map);
    groups::register(&mut map);
    access_points::register(&mut map);
    pfsense::register(&mut map);

    Procedures::new(map, Arc::new(State::default()))
}
