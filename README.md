# jsbm.dev

> Benchmark storage and redirecting API for [JSBenchmark.com](https://jsbenchmark.com)

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

To get up-and-running in development, do the following:

1. Clone `.env`

The example file is `.env.example`.

```sh
$ cp .env.example .env
```

2. Start postgres

Run `docker compose up --wait` to start postgres

3. Run migrations

Install the [`sqlx-cli`]() then run `sqlx migrate run`.

4. Start the server

`makers dev`

## Deployment

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
$ wrangler login # login to Cloudflare
$ wrangler deploy # deploy the worker
# for each key and value of PG_*, run the following command
# you can also do this on the Worker dashboard,
# but remember to click Entrypt!
$ wrangler secret put <key> # will open a prompt for the value
```

3. Profit.

Now, you can create your own shortcodes with the [API Documentation](#api) above.

You can test the worker by visiting the shortcode `default`.
