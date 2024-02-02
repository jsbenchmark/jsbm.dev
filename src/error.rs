use std::error::Error as StdError;
use std::fmt::Display;

use axum::response::{Html, IntoResponse, Json, Response};
use http::StatusCode;
use indoc::formatdoc;
use serde::Serialize;
use serde_json::json;

pub type BoxDynError = Box<dyn StdError + Send + Sync>;

/// Creates a generic JSON response with the given status code and message.
pub fn create_error_json<Message>(status: StatusCode, message: Message) -> Response
where Message: Serialize {
	let canonical_reason = status.canonical_reason().unwrap_or("Unknown");

	let source = json!({
		"status": canonical_reason,
		"message": message
	});

	(status, Json(source)).into_response()
}

/// Creates a generic HTML response with the given status code and message.
pub fn create_html_page<Message>(status: StatusCode, message: Message) -> Response
where Message: Display {
	let canonical_reason = status.canonical_reason().unwrap_or("Unknown");

	let source = formatdoc! {r#"
		<!DOCTYPE html>
		<html>

		<head>
			<title>{canonical_reason}</title>
			<style>
				body {{
					width: 35em;
					margin: 0 auto;
					font-family: Tahoma, Verdana, Arial, sans-serif;
				}}
			</style>
		</head>

		<body>
			<h1>{status}</h1>
			<hr>
			<p style="font-size: large;">{message}</p>
		</body>

		</html>
	"#};

	(status, Html(source)).into_response()
}
