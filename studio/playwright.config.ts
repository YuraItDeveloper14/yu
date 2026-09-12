import { defineConfig } from '@playwright/test';

// YU_URL points the test at a deployed Studio; without it Playwright starts the preview server.
// YU_CHROMIUM may point at a Chromium binary when Playwright's own browser isn't installed.
const url = process.env.YU_URL ?? 'http://localhost:4180';

export default defineConfig({
  testDir: 'tests',
  testMatch: /.*\.spec\.ts/,
  timeout: 60_000,
  use: {
    baseURL: url,
    launchOptions: process.env.YU_CHROMIUM ? { executablePath: process.env.YU_CHROMIUM } : {},
  },
  webServer: process.env.YU_URL
    ? undefined
    : { command: 'npx vite preview --port 4180 --strictPort', url, timeout: 60_000 },
});
