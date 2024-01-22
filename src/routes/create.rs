use base64::engine::general_purpose::URL_SAFE;
use base64::Engine as _;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use svix_ksuid::{KsuidLike, KsuidMs};
use worker::*;

use crate::error::create_error_json;
use crate::model::{create_database, ShortlinkMode};
use crate::ser::{BenchmarkState, ReplState};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum CreateShortcodeJsonBody {
	Benchmark(BenchmarkState),
	Repl(ReplState),
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

	let code = KsuidMs::new(None, None).to_string();
	let s = serde_json::to_vec(&body)?;
	let data = URL_SAFE.encode(&s);

	let db = create_database(&ctx.env).await?;

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
