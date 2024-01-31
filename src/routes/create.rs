use base64::engine::general_purpose::URL_SAFE;
use base64::Engine as _;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use worker::*;
use rand::distributions::{Alphanumeric, DistString};

use crate::error::create_error_json;
use crate::model::{create_database, ShortlinkMode};
use crate::ser::{BenchmarkState, ReplState};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CreateShortcodeJsonBody {
	Benchmark(BenchmarkState),
	Repl(ReplState),
}

fn generate_code() -> String {
	let string = Alphanumeric.sample_string(&mut rand::thread_rng(), 13);

	string
}

pub async fn create_shortcode(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
	let Ok(body) = req.json::<CreateShortcodeJsonBody>().await else {
		return create_error_json(
			StatusCode::BAD_REQUEST,
			"Body did not match benchmark or repl save schema",
		);
	};

	let mode = match body {
		CreateShortcodeJsonBody::Benchmark(_) => ShortlinkMode::Benchmark,
		CreateShortcodeJsonBody::Repl(_) => ShortlinkMode::Repl,
	};

	let s = serde_json::to_vec(&body)?;
	let data = URL_SAFE.encode(&s);

	let db = create_database(&ctx.env).await?;

	// Generate a unique shortcode.
	let mut attempts = 0;
	let max_attempts = 10;
	let mut code = generate_code();

	while let Ok(Some(_)) = db
		.query_opt("select code from shortcode where code = $1", &[&code])
		.await
	{
		code = generate_code();
		attempts += 1;
		if attempts >= max_attempts {
			return create_error_json(
				StatusCode::INTERNAL_SERVER_ERROR,
				"Failed to generate unique shortcode",
			);
		}
	}

	if let Err(err) = db
		.execute(
			"insert into shortcode(code, mode, data) values ($1, $2, $3);",
			&[&code, &mode.as_i32(), &data],
		)
		.await
	{
		console_error!("Failed to insert shortcode: {err:#?}");
		return create_error_json(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create shortcode");
	};

	Response::from_json(&serde_json::json!({
		"code": code,
	}))
}
