use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use base64::engine::general_purpose::URL_SAFE;
use base64::Engine;
use http::StatusCode;
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};

use crate::error::create_error_json;
use crate::model::ShortlinkMode;
use crate::ser::{BenchmarkState, ReplState};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateShortcodeJsonBody {
	Benchmark(BenchmarkState),
	Repl(ReplState),
}

#[axum::debug_handler]
pub async fn create_code(
	State(state): State<Arc<AppState>>,
	Json(body): Json<CreateShortcodeJsonBody>,
) -> impl IntoResponse {
	let mode = match body {
		CreateShortcodeJsonBody::Benchmark(_) => ShortlinkMode::Benchmark,
		CreateShortcodeJsonBody::Repl(_) => ShortlinkMode::Repl,
	};

	let Ok(s) = serde_json::to_vec(&body) else {
		return create_error_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize body").into_response();
	};
	let data = URL_SAFE.encode(&s);

	// Limit data size to 100KB.
	if data.bytes().len() > 100_000 {
		return create_error_json(StatusCode::PAYLOAD_TOO_LARGE, "Payload too large").into_response();
	}

	#[inline]
	fn generate_code() -> String {
		Alphanumeric.sample_string(&mut rand::thread_rng(), 13)
	}

	let mut attempts = 0;
	let max_attempts = 10;
	let mut code = generate_code();

	while let Ok(Some(_)) = sqlx::query!("select code from shortcode where code = $1", &code)
		.fetch_optional(&*state.db)
		.await
	{
		code = generate_code();
		attempts += 1;
		if attempts >= max_attempts {
			return create_error_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate unique shortcode")
				.into_response();
		}
	}

	if let Err(err) = sqlx::query!(
		"insert into shortcode(code, mode, data) values ($1, $2, $3)",
		&code,
		&mode.as_i32(),
		&data
	)
	.execute(&*state.db)
	.await
	{
		tracing::error!("Failed to insert shortcode into database: {:?}", err);
		return create_error_json(
			StatusCode::INTERNAL_SERVER_ERROR,
			"Failed to insert shortcode into database",
		)
		.into_response();
	}

	(StatusCode::CREATED, Json(serde_json::json!({ "code": code }))).into_response()
}
