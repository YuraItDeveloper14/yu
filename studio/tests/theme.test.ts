import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const css = readFileSync(new URL('../src/theme.css', import.meta.url), 'utf8');

/** The `--name: #rrggbb;` tokens of the block that starts with `selector {`. */
function tokens(selector: string): Record<string, string> {
  const start = css.indexOf(`${selector} {`);
  assert.ok(start >= 0, `no ${selector} block`);
  const block = css.slice(start, css.indexOf('}', start));
  return Object.fromEntries(
    [...block.matchAll(/--([\w-]+):\s*(#[0-9a-f]{6});/gi)].map((m) => [m[1], m[2].toLowerCase()]),
  );
}

/** Relative luminance, as WCAG defines it. */
function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5].map((i) => {
    const c = parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [light, dark] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (light + 0.05) / (dark + 0.05);
}

/** The `--accent-soft: rgba(r, g, b, a)` of the block that starts with `selector {`. */
function softOf(selector: string): number[] {
  const start = css.indexOf(`${selector} {`);
  const block = css.slice(start, css.indexOf('}', start));
  const match = /--accent-soft:\s*rgba\((\d+),\s*(\d+),\s*(\d+),\s*([\d.]+)\)/.exec(block);
  assert.ok(match, `no --accent-soft in ${selector}`);
  return match.slice(1).map(Number);
}

/** A highlighted row: `soft` laid over the background `under`. */
function over(soft: number[], under: string): string {
  const [r, g, b, alpha] = soft;
  return `#${[r, g, b]
    .map((channel, i) => {
      const below = parseInt(under.slice(1 + i * 2, 3 + i * 2), 16);
      return Math.round(alpha * channel + (1 - alpha) * below).toString(16).padStart(2, '0');
    })
    .join('')}`;
}

const soft: Record<string, number[]> = {
  dark: softOf(':root'),
  light: softOf(":root[data-theme='light']"),
};

const dark = tokens(':root');
const themes: Record<string, Record<string, string>> = {
  dark,
  light: { ...dark, ...tokens(":root[data-theme='light']") },
};

for (const [name, theme] of Object.entries(themes)) {
  test(`${name} theme: every text colour is readable where it sits`, () => {
    const pairs: [string, string][] = [];
    for (const fg of ['text', 'muted', 'accent-text']) {
      for (const bg of ['page', 'surface', 'raised']) pairs.push([fg, bg]);
    }
    for (const fg of ['syntax-keyword', 'syntax-builtin', 'syntax-string', 'syntax-number', 'syntax-comment']) {
      pairs.push([fg, 'surface']);
    }
    pairs.push(['error', 'surface'], ['error', 'raised'], ['on-accent', 'accent']);
    for (const [fg, bg] of pairs) {
      assert.ok(theme[fg] && theme[bg], `${fg} or ${bg} is missing`);
      const ratio = contrast(theme[fg], theme[bg]);
      assert.ok(ratio >= 4.5, `${fg} on ${bg}: ${ratio.toFixed(2)}`);
    }
    assert.ok(contrast('#ffffff', theme.stop) >= 4.5, 'white on stop');
  });

  test(`${name} theme: text stays readable on a highlighted row`, () => {
    for (const fg of ['text', 'muted']) {
      for (const bg of ['page', 'surface']) {
        const ratio = contrast(theme[fg], over(soft[name], theme[bg]));
        assert.ok(ratio >= 4.5, `${fg} on a highlighted ${bg}: ${ratio.toFixed(2)}`);
      }
    }
  });
}

test('the logo green is the accent of both themes', () => {
  assert.equal(themes.dark.accent, '#279f7c');
  assert.equal(themes.light.accent, '#279f7c');
});
