// The card GitHub shows when someone shares the repository: docs/social-preview.png, 1280×640.
// Upload it in the repository's Settings → General → Social preview.
// YU_CHROMIUM may point at a Chromium binary when Playwright's own browser isn't installed.
import { chromium } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const here = (path) => new URL(path, import.meta.url);

/** A fontsource stylesheet with its files inlined, so the page needs no server. */
function fonts(pkg) {
  const dir = here(`../node_modules/@fontsource-variable/${pkg}/`);
  return readFileSync(new URL('index.css', dir), 'utf8').replace(
    /url\(\.\/files\/([^)]+)\)/g,
    (_, name) => `url(data:font/woff2;base64,${readFileSync(new URL(`files/${name}`, dir)).toString('base64')})`,
  );
}

const logo = readFileSync(here('../public/logo.svg'), 'utf8');
const flag = readFileSync(here('../../examples/flag.svg'), 'utf8');

const html = `<!doctype html>
<html lang="uk">
<head>
<meta charset="utf-8">
<style>
${fonts('inter')}
${fonts('jetbrains-mono')}
body {
  box-sizing: border-box;
  width: 1280px;
  height: 640px;
  margin: 0;
  padding: 0 72px;
  display: grid;
  grid-template-columns: 1fr 440px;
  gap: 48px;
  align-items: center;
  background: #0e1c17;
  color: #e3f2eb;
  font-family: 'Inter Variable', sans-serif;
}
.name { display: flex; align-items: center; gap: 24px; font-size: 104px; font-weight: 800; }
.name svg { width: 104px; height: 104px; border-radius: 20px; }
h1 { margin: 32px 0 14px; font-size: 46px; line-height: 1.2; font-weight: 700; }
p { margin: 0 0 40px; color: #93b8a9; font-size: 27px; }
code { color: #4cc59c; font: 600 26px 'JetBrains Mono Variable', monospace; }
.picture svg { display: block; width: 440px; height: auto; border-radius: 16px; }
</style>
</head>
<body>
<div>
  <div class="name">${logo}<span>Yu</span></div>
  <h1>Мова програмування,<br>яка говорить українською</h1>
  <p>A programming language that speaks Ukrainian</p>
  <code>yu-lang.vercel.app</code>
</div>
<div class="picture">${flag}</div>
</body>
</html>`;

const browser = await chromium.launch(process.env.YU_CHROMIUM ? { executablePath: process.env.YU_CHROMIUM } : {});
const page = await browser.newPage({ viewport: { width: 1280, height: 640 }, reducedMotion: 'reduce' });
await page.setContent(html);
await page.evaluate(() => document.fonts.ready);
await page.screenshot({ path: fileURLToPath(here('../../docs/social-preview.png')) });
await browser.close();
console.log('docs/social-preview.png');
