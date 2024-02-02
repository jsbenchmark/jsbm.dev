use std::env::var;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, PgPool};

use crate::BoxDynError;

/// Creates a new connection to the Postgres database.
pub async fn create_database() -> Result<Arc<PgPool>, BoxDynError> {
	let database_url = var("DATABASE_URL")?;

	let pool = PgPoolOptions::new()
		.max_connections(32)
		.min_connections(4)
		.acquire_timeout(Duration::from_secs(8))
		.idle_timeout(Duration::from_secs(8))
		.max_lifetime(Duration::from_secs(120));

	let mut opts: PgConnectOptions = database_url.parse()?;
	opts = opts.log_statements(tracing::log::LevelFilter::Debug);

	let db = pool.connect_with(opts).await?;

	Ok(Arc::new(db))
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum ShortlinkMode {
	Benchmark = 0,
	Repl = 1,
}

impl ShortlinkMode {
	pub fn as_i32(&self) -> i32 {
		match self {
			ShortlinkMode::Benchmark => 0,
			ShortlinkMode::Repl => 1,
		}
	}
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ShortlinkRow {
	pub code: String,
	pub data: String,
	pub mode: ShortlinkMode,
}
