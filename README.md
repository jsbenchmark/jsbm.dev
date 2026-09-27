# jsbm.dev

> Benchmark storage and redirecting Cloudflare Worker for [JSBenchmark.com](https://jsbenchmark.com)

## API

### `GET` `/:shortcode`

Redirects to the benchmark with the given `shortcode`.

Examples:

- [`https://jsbm.dev/default`](https://jsbm.dev/default)
- [`https://jsbm.dev/default-repl`](https://jsbm.dev/default-repl)

### `POST` `/api/shortcode`

Creates a new shortcode for the provided benchmark state. Must match the structure of a Benchmark or Repl state (see below).

<details>
<summary>Benchmark</summary>

```typescript
interface BenchmarkState {
  cases: {
    id: string;
    code: string;
    name: string;
  }[];
  config: {
    name: string;
    runtime?: "worker" | "dom";
    benchmarkMode?: "quick" | "standard" | "extended";
    setupHtml?: string;
    parallel: boolean;
    globalTestConfig: {
      dependencies: {
        url: string;
        name: string;
        esm: boolean;
      }[];
    };
    dataCode: string;
  };
}
```

</details>

<details>
<summary>Repl</summary>

```typescript
interface ReplState {
  config: {
    name: string;
    runtime?: "worker" | "dom";
    setupHtml?: string;
    test: {
      dependencies: {
        url: string;
        name: string;
        esm: boolean;
      }[];
      code: string;
    };
  };
}
```

</details>

## Response

```json
{ "code": "2bEXvzJfZWuvetvRdkEUXwOkCVd" }
```

| Field  | Type     | Description                      |
| ------ | -------- | -------------------------------- |
| `code` | `string` | The shortcode for the benchmark. |

## Development

Install Rust through `rustup` and Node.js 24 or newer. `rust-toolchain.toml`
pins the compiler and WebAssembly target used locally and in CI. Linux builds
also need a C compiler, `pkg-config`, and OpenSSL development headers.

To get up-and-running in development, do the following:

1. Setup Wrangler

```bash
$ npm install --no-save --package-lock=false wrangler@4.142.0
```

Then, run `npx wrangler login` and follow the instructions.

1. Clone `.dev.vars`

The example file is prefixed with an underscore.

```sh
$ cp _.dev.vars .dev.vars
```

1. Start postgres (and pgbouncer)

Run `docker compose up --wait` to start postgres and pgbouncer.
Pgbouncer is used to pool connections to the database, which is required for serverless.
The migrations are ran automatically.

Neon, the database provider this project recommends in production, has a native pooling option, so this is only required for development.

## Verification

With Docker running, run the full test suite from the repository root:

```sh
./scripts/test.sh
```

CI uses the same command.

## Deployment

For an existing installation, push to `main` to run validation and deploy through
GitHub Actions (requires the `CLOUDFLARE_API_TOKEN` repository secret). Pull requests
run the same checks without deploying. To deploy locally after verification, run
`npx wrangler deploy --keep-vars`; this preserves dashboard variables and existing
secrets.

The following database setup is only needed for a new installation.

To deploy this project, do the following:

1. Create a database

...on https://neon.tech. Make sure you their pooling for serverless option.
Then, go to the SQL Editor tab and add the migrations in order from the `migrations` folder.

Then, deconstruct the database URL into the following variables.
You're gonna need this later.
It might be a good idea to do this in `.dev.vars` and comment out each line -- just so you have somewhere to copy and paste from.

```config
# PG_USER = ""
# PG_PASSWORD = ""
# PG_HOST = ""
# PG_DATABASE = ""
```

2. Deploy the worker

```console
$ npx wrangler login # login to Cloudflare
$ npx wrangler deploy --keep-vars # deploy the worker
# for each key and value of PG_*, run the following command
# you can also do this on the Worker dashboard,
# but remember to click Entrypt!
$ npx wrangler secret put <key> # will open a prompt for the value
```

3. Profit.

Now, you can create your own shortcodes with the [API Documentation](#api) above.

You can test the worker by visiting the shortcode `default`.
