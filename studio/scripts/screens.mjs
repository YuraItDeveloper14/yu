// Screenshots of the running Studio (start `npm run preview` first): dark with a menu open,
// light with the library, and a phone.
// YU_CHROMIUM may point at a Chromium binary when Playwright's own browser isn't installed.
import { chromium } from '@playwright/test';
import { readFileSync } from 'node:fs';

const url = process.env.YU_URL ?? 'http://localhost:4180';
const house = readFileSync(new URL('../../examples/house.yu', import.meta.url), 'utf8');
const browser = await chromium.launch(
  process.env.YU_CHROMIUM ? { executablePath: process.env.YU_CHROMIUM } : {},
);
const shots = [
  { name: 'dark', viewport: { width: 1440, height: 900 }, theme: 'dark', side: 'examples', menu: true },
  { name: 'light', viewport: { width: 1440, height: 900 }, theme: 'light', side: 'library', menu: false },
  { name: 'phone', viewport: { width: 390, height: 844 }, theme: 'dark', side: 'none', menu: false },
];
for (const shot of shots) {
  const page = await browser.newPage({ viewport: shot.viewport });
  page.on('pageerror', (error) => console.error(`${shot.name}: ${error.message}`));
  await page.addInitScript(
    ({ theme, side, code }) => {
      localStorage.setItem('yu-theme', theme);
      localStorage.setItem('yu-side', side);
      localStorage.setItem('yu-code', code);
      localStorage.setItem('yu-file', 'house.yu');
    },
    { theme: shot.theme, side: shot.side, code: house },
  );
  await page.goto(url);
  await page.locator('.cm-content').waitFor();
  await page.locator('#run').click();
  await page.locator('#picture svg').waitFor();
  await page.waitForTimeout(4500);
  if (shot.menu) {
    await page.locator('#menus [role="menuitem"]').first().click();
    await page.waitForTimeout(300);
  }
  const isolated = await page.evaluate(() => crossOriginIsolated);
  await page.screenshot({
    path: `test-results/screens/${shot.name}.png`,
    fullPage: shot.name === 'phone',
  });
  console.log(`${shot.name}: cross-origin isolated = ${isolated}`);
  await page.close();
}
await browser.close();
