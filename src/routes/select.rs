use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use http::StatusCode;

use crate::error::create_html_page;
use crate::model::ShortlinkMode;
use crate::AppState;

#[axum::debug_handler]
pub async fn get_code(State(state): State<Arc<AppState>>, Path(code): Path<String>) -> impl IntoResponse {
	let result = match sqlx::query!(
		r#"select data, mode as "mode: ShortlinkMode" from shortcode WHERE code = $1"#,
		&code
	)
	.fetch_optional(&*state.db)
	.await
	{
		Ok(Some(result)) => result,
		Ok(None) => return create_html_page(StatusCode::NOT_FOUND, "Shortcode not found".to_string()).into_response(),
		Err(e) => {
			tracing::error!("Error querying database: {e:#?}");
			return create_html_page(StatusCode::INTERNAL_SERVER_ERROR, "An unknown error occurred").into_response();
		}
	};

	let frontend_url = &state.frontend_url;
	let data = &result.data;
	let url = match result.mode {
		ShortlinkMode::Benchmark => format!("{frontend_url}/#{data}"),
		ShortlinkMode::Repl => format!("{frontend_url}/repl/#{data}"),
	};

	Redirect::permanent(&url).into_response()
}
