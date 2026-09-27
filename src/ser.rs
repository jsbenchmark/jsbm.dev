use serde::{Deserialize, Serialize};

/// A state for a Benchmark.
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkState {
	pub cases: Vec<BenchmarkCase>,
	pub config: BenchmarkConfig,
}

/// One of the test cases for a Benchmark.
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkCase {
	pub id: String,
	pub code: String,
	pub name: String,
	pub r#async: Option<bool>,
	pub dependencies: Option<Vec<Dependency>>,
}

/// The configuration for a Benchmark.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkConfig {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub runtime: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub benchmark_mode: Option<String>,
	pub parallel: bool,
	pub global_test_config: GlobalTestConfig,
	pub data_code: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub setup_html: Option<String>,
}

/// The dependencies for a Benchmark.
#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalTestConfig {
	pub dependencies: Vec<Dependency>,
}

/// A dependency to import from a CDN, usually jsDelivr.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
	pub url: String,
	pub name: String,
	pub esm: bool,
}

/// A state for a Repl.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplState {
	pub config: ReplConfig,
}

/// The configuration for a Repl.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplConfig {
	pub name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub runtime: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub setup_html: Option<String>,
	pub test: ReplTest,
}

/// The test for a Repl.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplTest {
	pub dependencies: Vec<Dependency>,
	pub code: String,
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::json;

	#[test]
	fn benchmark_sharing_preserves_runtime_html_and_mode() {
		let legacy = json!({
			"cases": [{ "id": "case-1", "name": "Read text", "code": "return 1", "async": false, "dependencies": [] }],
			"config": {
				"name": "Benchmark", "parallel": false,
				"globalTestConfig": { "dependencies": [] }, "dataCode": "const data = {}"
			}
		});

		for runtime in [None, Some("worker"), Some("dom")] {
			let mut payload = legacy.clone();
			if let Some(runtime) = runtime {
				payload["config"]["runtime"] = json!(runtime);
				payload["config"]["setupHtml"] = json!("<div id=\"fixture\">Hello & Grüß dich 🌍</div>\n");
				payload["config"]["benchmarkMode"] = json!("extended");
			}

			let state: BenchmarkState = serde_json::from_value(payload.clone()).unwrap();
			let saved: serde_json::Value = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
			assert_eq!(saved, payload);
		}
	}

	#[test]
	fn repl_sharing_preserves_runtime_and_html() {
		let legacy = json!({
			"config": {
				"name": "REPL", "test": { "dependencies": [], "code": "return 1" }
			}
		});

		for runtime in [None, Some("worker"), Some("dom")] {
			let mut payload = legacy.clone();
			if let Some(runtime) = runtime {
				payload["config"]["runtime"] = json!(runtime);
				payload["config"]["setupHtml"] = json!("<div id=\"fixture\">Hello & Grüß dich 🌍</div>\n");
			}

			let state: ReplState = serde_json::from_value(payload.clone()).unwrap();
			let saved: serde_json::Value = serde_json::from_slice(&serde_json::to_vec(&state).unwrap()).unwrap();
			assert_eq!(saved, payload);
		}
	}
}
