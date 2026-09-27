import assert from 'node:assert/strict';
import { after, before, describe, test } from 'node:test';
import { startWorker, WORKER_ORIGIN } from './helpers/worker.mjs';

const FRONTEND_ORIGIN = 'https://frontend.example';
const SETUP_HTML = '<div id="fixture">hello world</div>\n';

let worker;

before(async () => {
  worker = await startWorker();
});

after(async () => {
  await worker?.stop();
});

function createBenchmarkPayload(config = {}) {
  return {
    cases: [
      {
        id: 'case-1',
        name: 'Read text',
        code: 'return 1',
        async: false,
        dependencies: [],
      },
    ],
    config: {
      name: 'Benchmark',
      parallel: false,
      dataCode: 'return {}',
      globalTestConfig: {
        dependencies: [],
      },
      ...config,
    },
  };
}

function createReplPayload(config = {}) {
  return {
    config: {
      name: 'REPL',
      test: {
        code: 'return 1',
        dependencies: [],
      },
      ...config,
    },
  };
}

function request(path, options = {}) {
  return fetch(new URL(path, WORKER_ORIGIN), {
    redirect: 'manual',
    signal: AbortSignal.timeout(5_000),
    ...options,
  });
}

function postState(payload) {
  return request('/api/shortcode', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  });
}

async function createShortlink(payload) {
  const response = await postState(payload);
  assert.equal(response.status, 200, worker.logs());

  const { code } = await response.json();
  assert.match(code, /^[A-Za-z0-9]{13}$/);
  return code;
}

async function readSharedState(code, expectedPath) {
  const response = await request(`/${code}`);
  assert.equal(response.status, 302, worker.logs());

  const location = new URL(response.headers.get('location'));
  assert.equal(location.origin, FRONTEND_ORIGIN);
  assert.equal(location.pathname, expectedPath);

  const encodedState = location.hash.slice(1);
  const decodedState = Buffer.from(encodedState, 'base64url').toString('utf8');
  return JSON.parse(decodedState);
}

describe('benchmark sharing', () => {
  test('preserves legacy state without runtime fields', async () => {
    const payload = createBenchmarkPayload();

    const code = await createShortlink(payload);
    const restoredState = await readSharedState(code, '/');

    assert.deepEqual(restoredState, payload);
  });

  for (const runtime of ['worker', 'dom']) {
    test(`preserves ${runtime} runtime, HTML setup, and benchmark mode`, async () => {
      const payload = createBenchmarkPayload({
        runtime,
        setupHtml: SETUP_HTML,
        benchmarkMode: 'extended',
      });

      const code = await createShortlink(payload);
      const restoredState = await readSharedState(code, '/');

      assert.deepEqual(restoredState, payload);
    });
  }
});

describe('REPL sharing', () => {
  test('preserves legacy state without runtime fields', async () => {
    const payload = createReplPayload();

    const code = await createShortlink(payload);
    const restoredState = await readSharedState(code, '/repl/');

    assert.deepEqual(restoredState, payload);
  });

  for (const runtime of ['worker', 'dom']) {
    test(`preserves ${runtime} runtime and HTML setup`, async () => {
      const payload = createReplPayload({
        runtime,
        setupHtml: SETUP_HTML,
      });

      const code = await createShortlink(payload);
      const restoredState = await readSharedState(code, '/repl/');

      assert.deepEqual(restoredState, payload);
    });
  }
});

test('each save receives a different shortcode, even for identical state', async () => {
  const payload = createBenchmarkPayload();

  const firstCode = await createShortlink(payload);
  const secondCode = await createShortlink(payload);

  assert.notEqual(firstCode, secondCode);
});

describe('existing database rows', () => {
  const savedLinks = [
    { code: 'default', path: '/' },
    { code: 'default-repl', path: '/repl/' },
  ];

  for (const { code, path } of savedLinks) {
    test(`${code} still redirects to its saved state`, async () => {
      const state = await readSharedState(code, path);

      assert.equal(state.config.name, 'Basic example');
    });
  }
});

describe('error responses', () => {
  test('unknown shortcode returns 404', async () => {
    const response = await request('/missing-shortcode');

    assert.equal(response.status, 404);
  });

  test('invalid state returns the existing JSON error', async () => {
    const response = await postState({ config: {} });
    const error = await response.json();

    assert.equal(response.status, 400);
    assert.deepEqual(error, {
      status: 'Bad Request',
      message: 'Body did not match benchmark or repl save schema',
    });
  });

  test('malformed JSON returns 400', async () => {
    const response = await request('/api/shortcode', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{',
    });

    assert.equal(response.status, 400);
  });

  test('HTML setup counts toward the payload size limit', async () => {
    const payload = createBenchmarkPayload({
      runtime: 'dom',
      setupHtml: 'x'.repeat(100_000),
    });

    const response = await postState(payload);
    const error = await response.json();

    assert.equal(response.status, 413);
    assert.equal(error.message, 'Payload too large');
  });
});
