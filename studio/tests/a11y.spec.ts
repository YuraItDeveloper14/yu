import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';
import { watchEachPage } from './watch.ts';

watchEachPage();

/** What axe finds against WCAG 2.0 and 2.1, levels A and AA, as `rule: element` lines. */
async function violations(page: Page): Promise<string[]> {
  const result = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']).analyze();
  return result.violations.flatMap((rule) => rule.nodes.map((node) => `${rule.id}: ${node.target.join(' ')}`));
}

for (const theme of ['dark', 'light']) {
  test(`the Studio passes axe in the ${theme} theme`, async ({ page }) => {
    await page.addInitScript((picked) => localStorage.setItem('yu-theme', picked), theme);
    await page.goto('/');
    await expect(page.locator('.cm-content')).toBeVisible();
    expect(await violations(page)).toEqual([]);
  });

  test(`a book chapter passes axe in the ${theme} theme`, async ({ page }) => {
    await page.addInitScript((picked) => localStorage.setItem('yu-theme', picked), theme);
    await page.goto('/book/uk/03-conditions/');
    expect(await violations(page)).toEqual([]);
  });
}

test('the reference chapter passes axe', async ({ page }) => {
  await page.goto('/book/en/11-words/');
  expect(await violations(page)).toEqual([]);
});

test('a chapter at phone width passes axe', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/book/uk/08-turtle/');
  expect(await violations(page)).toEqual([]);
});
