// Builds the Yu core as WebAssembly and copies it to public/, where the page loads it from.
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const studio = join(dirname(fileURLToPath(import.meta.url)), '..');
const root = join(studio, '..');
execFileSync(
  'cargo',
  ['build', '--release', '--target', 'wasm32-unknown-unknown', '-p', 'yu-wasm'],
  { cwd: root, stdio: 'inherit' },
);
mkdirSync(join(studio, 'public'), { recursive: true });
copyFileSync(
  join(root, 'target', 'wasm32-unknown-unknown', 'release', 'yu_wasm.wasm'),
  join(studio, 'public', 'yu_wasm.wasm'),
);
console.log('public/yu_wasm.wasm is ready');
