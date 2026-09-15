import { expect, test, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
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

const opened = (page: Page, tab: 'picture' | 'terminal') =>
  expect(page.locator(`#tab-${tab}`)).toHaveAttribute('aria-selected', 'true');

test('the page is cross-origin isolated, so запитай can wait', async ({ page }) => {
  await page.goto('/');
  expect(await page.evaluate(() => crossOriginIsolated)).toBe(true);
});

test('the star example from the sidebar draws and opens Picture', async ({ page }) => {
  await page.goto('/');
  await page.locator('[data-example="house"]').click();
  await expect(page.locator('#file-name')).toHaveText('house.yu');
  await page.locator('[data-example="star"]').click();
  await expect(page.locator('#file-name')).toHaveText('star.yu');
  await page.locator('#run').click();
  await expect(page.locator('#picture svg polygon')).toHaveCount(1);
  await opened(page, 'picture');
  await expect(page.locator('#st-state')).toHaveText('Готово');
});

test('a program that only prints opens Terminal', async ({ page }) => {
  await open(page, 'скажи("Привіт")');
  await page.locator('#run').click();
  await expect(page.locator('#terminal')).toContainText('› yu програма.yu');
  await expect(page.locator('#terminal')).toContainText('Привіт');
  await opened(page, 'terminal');
});

test('an error opens Terminal and is underlined in the code', async ({ page }) => {
  await open(page, 'бал = 1\nскаж(бал)');
  await page.locator('#run').click();
  await expect(page.locator('#terminal .error')).toContainText('невідома назва «скаж»');
  await opened(page, 'terminal');
  await expect(page.locator('.cm-lintRange-error')).toHaveText('скаж');
});

test('запитай waits for an answer typed in Terminal', async ({ page }) => {
  await open(page, 'ім\'я = запитай("Як тебе звати?")\nскажи("Привіт, " + ім\'я)');
  await page.locator('#run').click();
  const input = page.locator('#terminal .ask input');
  await input.fill('Юрій');
  await input.press('Enter');
  await expect(page.locator('#terminal')).toContainText('Привіт, Юрій');
});

test('Stop ends an endless loop', async ({ page }) => {
  await open(page, 'поки так:\n    x = 1');
  await page.locator('#run').click();
  await expect(page.locator('#run')).toContainText('Стоп');
  await page.locator('#run').click();
  await expect(page.locator('#st-state')).toHaveText('Зупинено');
  await expect(page.locator('#run')).toContainText('Запустити');
});

test('English switches the interface and the errors', async ({ page }) => {
  await open(page, 'скаж(1)');
  await page.locator('#st-lang').click();
  await expect(page.locator('#run')).toContainText('Run');
  await expect(page.locator('#menus')).toContainText('File');
  await page.locator('#run').click();
  await expect(page.locator('#terminal .error')).toContainText("unknown name 'скаж'");
});

test('the File menu lists Save with its keys, and Escape closes it', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('menuitem', { name: 'Файл' }).click();
  const save = page.locator('.menu [data-command="save"]');
  await expect(save).toContainText('Зберегти');
  await expect(save).toContainText('Ctrl+S');
  await page.keyboard.press('Escape');
  await expect(page.locator('.menu')).toBeHidden();
});

test('Ctrl+S saves to the chosen file, and the next Ctrl+S writes there without asking', async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>;
    const writes: string[] = [];
    let asked = 0;
    w.yuWrites = writes;
    w.yuAsked = () => asked;
    w.showSaveFilePicker = async (options: { suggestedName: string }) => {
      asked += 1;
      return {
        name: options.suggestedName,
        getFile: async () => new File([], options.suggestedName),
        createWritable: async () => ({
          write: async (data: string) => void writes.push(data),
          close: async () => {},
        }),
      };
    };
  });
  await page.goto('/');
  await page.locator('.cm-content').click();
  await page.keyboard.press('Control+End');
  await page.keyboard.type('\n# моя зірка');
  await expect(page.locator('#file-dot')).toBeVisible();
  await page.keyboard.press('Control+S');
  await expect(page.locator('#file-dot')).toBeHidden();
  await page.keyboard.type('!');
  await page.keyboard.press('Control+S');
  const writes = () => page.evaluate(() => (window as unknown as { yuWrites: string[] }).yuWrites);
  await expect.poll(async () => (await writes()).length).toBe(2);
  expect((await writes())[1]).toContain('# моя зірка!');
  expect(await page.evaluate(() => (window as unknown as { yuAsked(): number }).yuAsked())).toBe(1);
});

test('without the file API, Ctrl+S downloads the program and Ctrl+O opens a chosen file', async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>;
    w.showSaveFilePicker = undefined;
    w.showOpenFilePicker = undefined;
  });
  await page.goto('/');
  await page.locator('.cm-content').click();
  const downloading = page.waitForEvent('download');
  await page.keyboard.press('Control+S');
  const file = await downloading;
  expect(file.suggestedFilename()).toBe('star.yu');
  expect(readFileSync(await file.path(), 'utf8')).toContain('почни_заливку()');
  const choosing = page.waitForEvent('filechooser');
  await page.keyboard.press('Control+O');
  const chooser = await choosing;
  await chooser.setFiles({
    name: 'мій.yu',
    mimeType: 'text/plain',
    buffer: Buffer.from('скажи("з файлу")\n'),
  });
  await expect(page.locator('.cm-content')).toContainText('скажи("з файлу")');
  await expect(page.locator('#file-name')).toHaveText('мій.yu');
});

test('the library finds коло, runs its example and inserts it', async ({ page }) => {
  await page.goto('/');
  await page.locator('#act-library').click();
  await page.locator('#library-search').fill('коло');
  const entry = page.locator('.entry[data-word="коло"]');
  await entry.locator('summary').click();
  await entry.getByRole('button', { name: 'Спробувати' }).click();
  await expect(entry.locator('.result img')).toBeVisible();
  await expect(entry.locator('.result img')).toHaveAttribute('src', /%3Ccircle/);
  await entry.getByRole('button', { name: 'Вставити' }).click();
  await expect(page.locator('.cm-content')).toContainText('коло(300, 200, 80)');
  await expect(page.locator('#file-dot')).toBeVisible();
});
