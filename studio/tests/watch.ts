// What no browser test may leave behind: an uncaught error, or anything the site's
// Content-Security-Policy blocked.
import { expect, test } from '@playwright/test';

export function watchEachPage(): void {
  let problems: string[] = [];

  test.beforeEach(async ({ page }) => {
    problems = [];
    page.on('pageerror', (error) => problems.push(error.message));
    page.on('console', (message) => {
      if (message.text().includes('Content Security Policy')) problems.push(message.text());
    });
    await page.addInitScript(() => {
      document.addEventListener('securitypolicyviolation', (event) => {
        console.error(`Content Security Policy blocked ${event.blockedURI} (${event.violatedDirective})`);
      });
    });
  });

  test.afterEach(() => {
    expect(problems).toEqual([]);
  });
}
