import { test } from 'node:test';
import assert from 'node:assert/strict';
import { decode, encode, tokenFromHash } from '../src/share.ts';

test('a program survives the trip through a link', async () => {
  const code = 'для i від 1 до 3:\n    скажи("Привіт, " + i)  # ім\'я\n';
  const token = await encode(code);
  assert.match(token, /^[A-Za-z0-9_-]+$/);
  assert.equal(await decode(token), code);
  assert.equal(tokenFromHash(`#code=${token}`), token);
});

test('a broken link is an error, not garbage', async () => {
  await assert.rejects(decode('bm90LWRlZmxhdGU'));
  assert.equal(tokenFromHash('#інше'), null);
});
