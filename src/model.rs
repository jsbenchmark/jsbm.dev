use std::error::Error as StdError;
use std::result::Result as StdResult;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tokio_postgres::config::SslMode;
use tokio_postgres::types::{FromSql, Type};
use worker::postgres_tls::PassthroughTls;
use worker::*;

#[derive(Debug, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
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

	pub fn from_i32(i: i32) -> StdResult<Self, Box<dyn StdError + Sync + Send>> {
		match i {
			0 => Ok(ShortlinkMode::Benchmark),
			1 => Ok(ShortlinkMode::Repl),
			_ => Err("invalid shortlink mode".into()),
		}
	}
}

impl<'a> FromSql<'a> for ShortlinkMode {
	fn from_sql(ty: &Type, raw: &'a [u8]) -> StdResult<Self, Box<dyn StdError + Sync + Send>> {
		let i = i32::from_sql(ty, raw)?;

		ShortlinkMode::from_i32(i)
	}

	fn accepts(ty: &Type) -> bool {
		i32::accepts(ty)
	}
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ShortlinkRow {
	pub code: String,
	pub data: String,
	pub mode: ShortlinkMode,
}

/// Creates a new connection to the Postgres database.
pub async fn create_database(env: &Env) -> Result<tokio_postgres::Client> {
	let mut config = tokio_postgres::config::Config::new();
	config.user(&env.secret("PG_USER")?.to_string());
	config.password(&env.secret("PG_PASSWORD")?.to_string());

	if env
		.var("ENVIRONMENT")
		.map(|e| &e.to_string() == "production")
		.unwrap_or(false)
	{
		config.ssl_mode(SslMode::Require);
	}

	config.dbname(&env.secret("PG_DATABASE")?.to_string());

	let socket = Socket::builder()
		.secure_transport(SecureTransport::StartTls)
		.connect(&env.secret("PG_HOST")?.to_string(), 5432)?;
	let (client, connection) = config
		.connect_raw(socket, PassthroughTls)
		.await
		.map_err(|e| worker::Error::RustError(format!("tokio-postgres: {e:#?}")))?;

	wasm_bindgen_futures::spawn_local(async move {
		if let Err(error) = connection.await {
			console_error!("postgres connection error: {error:#?}");
		}
	});

	Ok(client)
}
