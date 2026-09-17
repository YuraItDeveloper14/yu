import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

// SharedArrayBuffer (how запитай waits for an answer) needs a cross-origin isolated page.
const isolation = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

/** The headers the site sends (public/vercel.json), so the preview and every test run on it get them too. */
function siteHeaders(): Record<string, string> {
  const site = JSON.parse(readFileSync(new URL('./public/vercel.json', import.meta.url), 'utf8')) as {
    headers: { headers: { key: string; value: string }[] }[];
  };
  return Object.fromEntries(site.headers[0].headers.map(({ key, value }) => [key, value]));
}

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
  // Hot reload needs a websocket and inline code, so `vite dev` sends only the isolation headers.
  server: { headers: isolation, fs: { allow: ['..'] } },
  preview: { headers: siteHeaders() },
  worker: { format: 'es' },
  build: {
    target: 'es2022',
    // Fonts stay files: the policy's font-src 'self' allows no data: URLs.
    assetsInlineLimit: 0,
    rollupOptions: {
      input: {
        main: fileURLToPath(new URL('./index.html', import.meta.url)),
        ...bookPages(),
      },
    },
  },
});
