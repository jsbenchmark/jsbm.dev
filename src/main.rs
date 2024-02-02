use std::env::var;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use error::{create_html_page, BoxDynError};
use http::StatusCode;
use model::create_database;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};

mod error;
mod model;
mod routes;
mod ser;

use crate::routes::create::create_code;
use crate::routes::select::get_code;

pub struct AppState {
	db: Arc<PgPool>,
	frontend_url: String,
}

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
	let port = var("PORT")
		.map(|p| p.parse::<u16>().expect("failed to parse port"))
		.unwrap_or_else(|_| 47143);

	let frontend_url = var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

	Registry::default()
		.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug,tower_http=debug,axum=trace".into()))
		.with(fmt::layer())
		.init();

	let db = create_database().await.expect("failed to create database connection");
	let state = Arc::new(AppState { db, frontend_url });

	let app = Router::new()
		.route("/", get(|| async { create_html_page(StatusCode::OK, "Hello, world!") }))
		.route("/:code", get(get_code))
		.route("/api/shortcode", post(create_code))
		.with_state(state)
		.layer((
			TraceLayer::new_for_http(),
			TimeoutLayer::new(Duration::from_secs(10)),
			DefaultBodyLimit::max(1024 * 100 /* 100kb */),
		));

	let socket = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
	tracing::info!("Listening on http://{socket}");
	let listener = TcpListener::bind(socket).await.expect("failed to bind tcp listener");

	let _ = axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await;

	Ok(())
}

async fn shutdown_signal() {
	let ctrl_c = async {
		tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
			.expect("failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {},
		_ = terminate => {},
	}

	tracing::info!("signal received, starting graceful shutdown");
}
