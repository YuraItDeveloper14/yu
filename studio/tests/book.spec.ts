import { expect, test } from '@playwright/test';

let errors: string[] = [];

test.beforeEach(({ page }) => {
  errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
});

test.afterEach(() => {
  expect(errors).toEqual([]);
});

test('a chapter shows its heading and the whole book beside it', async ({ page }) => {
  await page.goto('/book/uk/01-start/');
  await expect(page.locator('h1')).toHaveText('Перша програма');
  await expect(page.locator('.chapters a')).toHaveCount(11);
  await expect(page.locator('.chapters a[aria-current="page"]')).toHaveText(/Перша програма/);
  await expect(page.locator('.book-foot a')).toHaveCount(1);
});

test('an example opens in the Studio with its code', async ({ page }) => {
  await page.goto('/book/uk/01-start/');
  const open = page.locator('.example .open').first();
  await expect(open).toHaveAttribute('href', /^\/#code=/);
  await open.click();
  await expect(page.locator('.cm-content')).toContainText('скажи("Привіт, світе!")');
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
