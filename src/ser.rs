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
}

/// The configuration for a Benchmark.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkConfig {
	pub name: String,
	pub parallel: bool,
	pub global_test_config: GlobalTestConfig,
	pub data_code: String,
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
pub struct ReplConfig {
	pub name: String,
	pub test: ReplTest,
}

/// The test for a Repl.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReplTest {
	pub dependencies: Vec<Dependency>,
	pub code: String,
}
