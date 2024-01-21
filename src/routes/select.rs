use http::StatusCode;
use worker::*;

use crate::error::create_html_page;
use crate::model::{create_database, ShortlinkMode};

pub async fn select_shortcode(_: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
	let Some(shortcode) = ctx.param("shortcode") else {
		return create_html_page(StatusCode::BAD_REQUEST, "No shortcode provided".to_string());
	};

	let db = create_database(&ctx.env).await?;
	let result = match db
		.query_opt("select data, mode from shortcode WHERE code = $1", &[shortcode])
		.await
	{
		Ok(Some(result)) => result,
		Ok(None) => return create_html_page(StatusCode::NOT_FOUND, "Shortcode not found".to_string()),
		Err(e) => {
			console_error!("Error querying database: {e:#?}");
			return create_html_page(StatusCode::INTERNAL_SERVER_ERROR, "An unknown error occurred");
		}
	};

	let encoded_data: &str = result.get("data");
	let mode: ShortlinkMode = result.get("mode");

	let fqdn = ctx.env.var("JSBENCHMARK_FQDN")?.to_string();
	let url = match mode {
		ShortlinkMode::Benchmark => Url::parse(&format!("https://{fqdn}/#{}", encoded_data))?,
		ShortlinkMode::Repl => Url::parse(&format!("https://{fqdn}/repl/#{}", encoded_data))?,
	};

	Response::redirect(url)
}
