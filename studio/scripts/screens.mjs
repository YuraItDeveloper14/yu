// Screenshots of the running Studio (start `npm run preview` first): dark, light and a phone.
// YU_CHROMIUM may point at a Chromium binary when Playwright's own browser isn't installed.
import { chromium } from '@playwright/test';

const url = process.env.YU_URL ?? 'http://localhost:4180';
const browser = await chromium.launch(
  process.env.YU_CHROMIUM ? { executablePath: process.env.YU_CHROMIUM } : {},
);
const shots = [
  { name: 'dark', viewport: { width: 1440, height: 900 }, theme: 'dark' },
  { name: 'light', viewport: { width: 1440, height: 900 }, theme: 'light' },
  { name: 'phone', viewport: { width: 390, height: 844 }, theme: 'dark' },
];
for (const shot of shots) {
  const page = await browser.newPage({ viewport: shot.viewport });
  page.on('pageerror', (error) => console.error(`${shot.name}: ${error.message}`));
  await page.addInitScript((theme) => localStorage.setItem('yu-theme', theme), shot.theme);
  await page.goto(url);
  await page.locator('.cm-content').waitFor();
  await page.locator('#examples').selectOption('house');
  await page.locator('#run').click();
  await page.locator('#picture svg').waitFor();
  await page.waitForTimeout(4500);
  const isolated = await page.evaluate(() => crossOriginIsolated);
  await page.screenshot({
    path: `test-results/screens/${shot.name}.png`,
    fullPage: shot.name === 'phone',
  });
  console.log(`${shot.name}: cross-origin isolated = ${isolated}`);
  await page.close();
}
await browser.close();
