import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { setTimeout as delay } from 'node:timers/promises';

export const WORKER_ORIGIN = 'http://127.0.0.1:8787';

const STARTUP_TIMEOUT_MS = 30_000;
const SHUTDOWN_TIMEOUT_MS = 5_000;
const POLL_INTERVAL_MS = 250;

function hasExited(child) {
  return child.exitCode !== null || child.signalCode !== null;
}

async function waitForWorker(child) {
  const deadline = Date.now() + STARTUP_TIMEOUT_MS;

  while (Date.now() < deadline) {
    if (hasExited(child)) {
      throw new Error('Worker exited before it was ready');
    }

    try {
      const response = await fetch(WORKER_ORIGIN, {
        signal: AbortSignal.timeout(500),
      });
      await response.body?.cancel();

      if (response.status === 200) {
        return;
      }
    } catch {
      // Connection failures are expected while the local server starts.
    }

    await delay(POLL_INTERVAL_MS);
  }

  throw new Error('Timed out waiting for the worker to start');
}

async function stopWorker(child) {
  if (hasExited(child)) {
    return;
  }

  const exited = once(child, 'exit', {
    signal: AbortSignal.timeout(SHUTDOWN_TIMEOUT_MS),
  });
  child.kill('SIGTERM');

  try {
    await exited;
  } catch (error) {
    if (error.name !== 'AbortError') {
      throw error;
    }

    const forcedExit = once(child, 'exit');
    child.kill('SIGKILL');
    await forcedExit;
  }
}

export async function startWorker() {
  const wranglerPath = process.env.WRANGLER_BIN ?? 'node_modules/wrangler/bin/wrangler.js';
  const args = [
    wranglerPath,
    'dev',
    '--local',
    '--config',
    'tests/wrangler.toml',
    '--ip',
    '127.0.0.1',
    '--port',
    '8787',
  ];
  if (process.env.JSBM_TEST_PG_HOST) {
    args.push('--var', `PG_HOST:${process.env.JSBM_TEST_PG_HOST}`);
  }

  const worker = spawn(process.execPath, args, {
    env: { ...process.env, WRANGLER_SEND_METRICS: 'false' },
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  let logs = '';
  function captureOutput(chunk) {
    logs += chunk.toString();
  }
  worker.stdout.on('data', captureOutput);
  worker.stderr.on('data', captureOutput);

  await once(worker, 'spawn');

  try {
    await waitForWorker(worker);
  } catch (error) {
    await stopWorker(worker);
    throw new Error(`${error.message}\n${logs}`, { cause: error });
  }

  return {
    logs: () => logs,
    stop: () => stopWorker(worker),
  };
}
