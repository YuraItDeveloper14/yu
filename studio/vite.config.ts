import { readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

// SharedArrayBuffer (how запитай waits for an answer) needs a cross-origin isolated page.
const isolation = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

/** Every page `npm run book` wrote, so Vite builds them beside the Studio. */
function bookPages(): Record<string, string> {
  const dir = fileURLToPath(new URL('./book', import.meta.url));
  let names: string[] = [];
  try {
    names = readdirSync(dir, { recursive: true })
      .map((name) => String(name).replace(/\\/g, '/'))
      .filter((name) => name.endsWith('index.html'));
  } catch {
    return {}; // the book has not been generated yet
  }
  return Object.fromEntries(
    names.map((name) => [`book-${name.replace(/\/?index\.html$/, '').replace(/\//g, '-') || 'home'}`, `${dir.replace(/\\/g, '/')}/${name}`]),
  );
}

export default defineConfig({
  server: { headers: isolation, fs: { allow: ['..'] } },
  preview: { headers: isolation },
  worker: { format: 'es' },
  build: {
    target: 'es2022',
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL('./index.html', import.meta.url)),
        ...bookPages(),
      },
    },
  },
});
