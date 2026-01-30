#!/usr/bin/env node

/**
 * Development server launcher with automatic port selection.
 * Finds an available port and starts both Vite and Tauri with matching configuration.
 */

import { createServer } from 'net';
import { spawn } from 'child_process';

const BASE_PORT = 5173;
const MAX_PORT_ATTEMPTS = 100;

/**
 * Check if a port is available
 */
function isPortAvailable(port) {
  return new Promise((resolve) => {
    const server = createServer();
    server.once('error', () => resolve(false));
    server.once('listening', () => {
      server.close();
      resolve(true);
    });
    server.listen(port, '127.0.0.1');
  });
}

/**
 * Find an available port starting from basePort
 */
async function findAvailablePort(basePort) {
  for (let i = 0; i < MAX_PORT_ATTEMPTS; i++) {
    const port = basePort + i;
    if (await isPortAvailable(port)) {
      return port;
    }
  }
  throw new Error(`No available port found in range ${basePort}-${basePort + MAX_PORT_ATTEMPTS}`);
}

async function main() {
  try {
    const port = await findAvailablePort(BASE_PORT);
    const devUrl = `http://localhost:${port}`;

    if (port !== BASE_PORT) {
      console.log(`\x1b[33mPort ${BASE_PORT} is in use, using port ${port}\x1b[0m`);
    }

    // Set environment variables for both Vite and Tauri
    const env = {
      ...process.env,
      VITE_DEV_PORT: String(port),
      TAURI_DEV_URL: devUrl,
    };

    // Spawn tauri dev (which will spawn vite via beforeDevCommand)
    const tauri = spawn('npm', ['run', 'tauri', 'dev'], {
      stdio: 'inherit',
      shell: true,
      env,
    });

    tauri.on('close', (code) => {
      process.exit(code ?? 0);
    });

    // Handle Ctrl+C gracefully
    process.on('SIGINT', () => {
      tauri.kill('SIGINT');
    });

    process.on('SIGTERM', () => {
      tauri.kill('SIGTERM');
    });

  } catch (error) {
    console.error('Failed to start development server:', error.message);
    process.exit(1);
  }
}

main();
