import { expect, test, type Page } from '@playwright/test';
import { encode } from '../src/share.ts';

let errors: string[] = [];

test.beforeEach(({ page }) => {
  errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
});

test.afterEach(() => {
  expect(errors).toEqual([]);
});

/** Opens the Studio with `code` in the editor, through a share link. */
async function open(page: Page, code: string): Promise<void> {
  await page.goto(`/#code=${await encode(code)}`);
  await expect(page.locator('.cm-content')).toContainText(code.split('\n')[0]);
}

test('the page is cross-origin isolated, so запитай can wait', async ({ page }) => {
  await page.goto('/');
  expect(await page.evaluate(() => crossOriginIsolated)).toBe(true);
});

test('an example runs and draws', async ({ page }) => {
  await page.goto('/');
  await page.locator('#examples').selectOption('star');
  await page.locator('#run').click();
  await expect(page.locator('#picture svg polygon')).toHaveCount(1);
  await expect(page.locator('#status')).toHaveText('Готово');
});

test('an error shows in the output and under the code', async ({ page }) => {
  await open(page, 'бал = 1\nскаж(бал)');
  await page.locator('#run').click();
  await expect(page.locator('#output .error')).toContainText('невідома назва «скаж»');
  await expect(page.locator('.cm-lintRange-error')).toHaveText('скаж');
});

test('запитай waits for an answer typed in the output', async ({ page }) => {
  await open(page, 'ім\'я = запитай("Як тебе звати?")\nскажи("Привіт, " + ім\'я)');
  await page.locator('#run').click();
  const input = page.locator('#output .ask input');
  await input.fill('Юрій');
  await input.press('Enter');
  await expect(page.locator('#output')).toContainText('Привіт, Юрій');
});

test('Stop ends an endless loop', async ({ page }) => {
  await open(page, 'поки так:\n    x = 1');
  await page.locator('#run').click();
  await expect(page.locator('#run')).toHaveText('Стоп');
  await page.locator('#run').click();
  await expect(page.locator('#status')).toHaveText('Зупинено');
  await expect(page.locator('#run')).toHaveText('Запустити');
});

test('English switches the interface and the errors', async ({ page }) => {
  await open(page, 'скаж(1)');
  await page.locator('#lang').click();
  await expect(page.locator('#run')).toHaveText('Run');
  await page.locator('#run').click();
  await expect(page.locator('#output .error')).toContainText("unknown name 'скаж'");
});
