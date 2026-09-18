import { expect, test } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { watchEachPage } from './watch.ts';

watchEachPage();

const site = JSON.parse(readFileSync(new URL('../public/vercel.json', import.meta.url), 'utf8')) as {
  headers: { source: string; headers: { key: string; value: string }[] }[];
};

for (const path of ['/', '/book/uk/01-start/', '/yu_wasm.wasm']) {
  test(`${path} comes with the site's security headers`, async ({ request }) => {
    const response = await request.get(path);
    expect(response.ok()).toBe(true);
    const headers = response.headers();
    expect(headers['content-security-policy']).toContain("script-src 'self' 'wasm-unsafe-eval'");
    for (const { key, value } of site.headers[0].headers) expect(headers[key.toLowerCase()]).toBe(value);
  });
}
