// Records ../docs/studio.gif from the running Studio (start `npm run preview` first):
// the star draws itself, then FizzBuzz prints in the terminal.
// YU_CHROMIUM may point at a Chromium binary when Playwright's own browser isn't installed.
import { chromium } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import { mkdirSync, rmSync, statSync } from 'node:fs';

const url = process.env.YU_URL ?? 'http://localhost:4180';
const frames = 'test-results/gif';
const gif = '../docs/studio.gif';
rmSync(frames, { recursive: true, force: true });
mkdirSync(frames, { recursive: true });

const browser = await chromium.launch(
  process.env.YU_CHROMIUM ? { executablePath: process.env.YU_CHROMIUM } : {},
);
const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
await page.goto(url);
await page.locator('.cm-content').waitFor();

let count = 0;
let filmed = 0;
/** Screenshots, one after another, for `ms` milliseconds. */
async function film(ms) {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    await page.screenshot({ path: `${frames}/${String(count).padStart(4, '0')}.png` });
    count += 1;
  }
  filmed += ms;
}

await film(700);
await page.locator('#run').click();
await film(5200);
await page.locator('[data-example="fizzbuzz"]').click();
await film(700);
await page.locator('#run').click();
await film(2400);
await browser.close();

const fps = Math.max(1, Math.round(count / (filmed / 1000)));
execFileSync(
  'ffmpeg',
  [
    '-y',
    '-loglevel',
    'error',
    '-framerate',
    String(fps),
    '-i',
    `${frames}/%04d.png`,
    '-vf',
    'scale=960:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse=dither=bayer:bayer_scale=5',
    gif,
  ],
  { stdio: 'inherit' },
);
console.log(`${gif}: ${count} frames at ${fps} fps, ${(statSync(gif).size / 1e6).toFixed(2)} MB`);
