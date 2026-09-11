import { defineConfig } from 'vite';

// SharedArrayBuffer (how запитай waits for an answer) needs a cross-origin isolated page.
const isolation = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

export default defineConfig({
  server: { headers: isolation, fs: { allow: ['..'] } },
  preview: { headers: isolation },
  worker: { format: 'es' },
  build: { target: 'es2022' },
});
