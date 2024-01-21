use error::create_html_page;
use http::StatusCode;
use routes::create::create_shortcode;
use routes::select::select_shortcode;
use utils::log_request;
use worker::*;

mod error;
mod model;
mod routes;
mod ser;
mod utils;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
	log_request(&req);
	utils::set_panic_hook();

	Router::new()
		.get("/", |_, _| {
			// todo: figure out what we wanna do here
			create_html_page(StatusCode::OK, "Hello, world!")
		})
		.get_async("/:shortcode", select_shortcode)
		.post_async("/api/shortcode", create_shortcode)
		.run(req, env)
		.await
}
