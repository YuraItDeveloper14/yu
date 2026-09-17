import { expect, test } from '@playwright/test';
import { watchEachPage } from './watch.ts';

watchEachPage();

test('a chapter shows its heading and the whole book beside it', async ({ page }) => {
  await page.goto('/book/uk/01-start/');
  await expect(page.locator('h1')).toHaveText('Перша програма');
  await expect(page.locator('.chapters a')).toHaveCount(11);
  await expect(page.locator('.chapters a[aria-current="page"]')).toHaveText(/Перша програма/);
  await expect(page.locator('.book-foot a')).toHaveCount(1);
  await expect(page.locator('meta[name="description"]')).toHaveAttribute(
    'content',
    'Yu працює просто в браузері: відкрий yu-lang.vercel.app — і можна писати. Встановлювати нічого не треба.',
  );
});

test('an example opens in the Studio with its code', async ({ page }) => {
  await page.goto('/book/uk/01-start/');
  const open = page.locator('.book-code .open').first();
  await expect(open).toHaveAttribute('href', /^\/#code=/);
  await open.click();
  await expect(page.locator('.cm-content')).toContainText('скажи("Привіт, світе!")');
});

test('«Відкрити в Студії» stands under the code, not over it', async ({ page }) => {
  await page.goto('/book/uk/01-start/');
  const example = page.locator('.book-code').nth(1);
  const code = await example.locator('pre').boundingBox();
  const open = await example.locator('.open').boundingBox();
  expect(code && open && open.y >= code.y + code.height).toBe(true);
});

test('an example that draws shows its picture', async ({ page }) => {
  await page.goto('/book/uk/08-turtle/');
  const pictures = page.locator('.book-result .picture');
  await expect(pictures).toHaveCount(7);
  const first = pictures.first();
  await first.scrollIntoViewIfNeeded();
  await expect.poll(() => first.evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth)).toBe(600);
});

test('on a phone the chapters fold away and nothing scrolls sideways', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 780 });
  await page.goto('/book/en/11-words/');
  await expect(page.locator('#book-search')).toBeVisible();
  await expect(page.locator('#book-other')).toBeVisible();
  await expect(page.locator('#book-theme')).toBeVisible();
  const list = page.locator('.chapters ol');
  await expect(list).toBeHidden();
  await page.locator('.chapters summary').click();
  await expect(list).toBeVisible();
  const sideways = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(sideways).toBe(0);
});

test('the theme picked in the Studio is the one the book opens with', async ({ page }) => {
  await page.goto('/');
  await page.locator('#st-theme').click();
  const theme = await page.evaluate(() => document.documentElement.dataset.theme);
  await page.goto('/book/uk/04-loops/');
  await expect(page.locator('html')).toHaveAttribute('data-theme', String(theme));
});

test('search finds a chapter and Enter opens it', async ({ page }) => {
  await page.goto('/book/uk/01-start/');
  await page.locator('#book-search').fill('цикл');
  await expect(page.locator('#book-results a').first()).toContainText('Цикли');
  await page.locator('#book-search').press('Enter');
  await expect(page).toHaveURL(/\/book\/uk\/04-loops\//);
});

test('the same chapter is one click away in English', async ({ page }) => {
  await page.goto('/book/uk/04-loops/');
  await page.locator('#book-other').click();
  await expect(page).toHaveURL(/\/book\/en\/04-loops\//);
  await expect(page.locator('h1')).toHaveText('Loops');
});

test('the Studio Help menu opens the book', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('menuitem', { name: 'Довідка' }).click();
  await expect(page.locator('.menu [data-command="book"]')).toContainText('Книга Yu');
});

test('/book/ opens the first chapter in the language the reader picked', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('yu-lang', 'en'));
  await page.goto('/book/');
  await expect(page).toHaveURL(/\/book\/en\/01-start\/$/);
});
