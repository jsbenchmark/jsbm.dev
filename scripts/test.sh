#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

compose=(docker compose
  --project-name "jsbm-tests-$$"
  --file "$repo_root/tests/compose.yml"
)

cleanup() {
  "${compose[@]}" down --volumes --rmi local
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

"${compose[@]}" up --build --exit-code-from tests
