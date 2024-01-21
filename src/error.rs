use std::fmt::Display;

use http::StatusCode;
use indoc::formatdoc;
use serde::Serialize;
use serde_json::json;
use worker::*;

/// Creates a generic JSON response with the given status code and message.
pub fn create_error_json<Message>(status: StatusCode, message: Message) -> worker::Result<Response>
where Message: Serialize {
	let canonical_reason = status.canonical_reason().unwrap_or("Unknown");

	let source = json!({
		"status": canonical_reason,
		"message": message
	});

	Ok(Response::from_json(&source)?.with_status(status.as_u16()))
}

/// Creates a generic HTML response with the given status code and message.
pub fn create_html_page<Message>(status: StatusCode, message: Message) -> worker::Result<Response>
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

	Ok(Response::from_html(source)?.with_status(status.as_u16()))
}
