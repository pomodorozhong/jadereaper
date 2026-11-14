// Smoke test: load the game in headless Chromium and fail on any console
// error, page error or failed request.
//
//     npm run smoke
//
// With WebGPU, it starts a run and plays a few seconds. Without WebGPU (common
// on CI machines with no GPU), it checks that the "Cannot start" screen shows
// the WebGPU reason instead, which is the expected outcome there.

import { chromium } from 'playwright';
import { serve } from './serve.mjs';

const PLAY_MS = 3000;
const TIMEOUT_MS = 20000;

const server = await serve(0);
const url = `http://localhost:${server.address().port}/`;
const browser = await chromium.launch({ args: ['--enable-unsafe-webgpu'] });
const problems = [];
let consoleErrors = [];
let outcome;

try {
  const page = await browser.newPage();
  page.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(msg.text());
  });
  page.on('pageerror', (err) => problems.push(`page error: ${err.message}`));
  page.on('requestfailed', (req) => problems.push(`request failed: ${req.url()} (${req.failure()?.errorText})`));
  page.on('response', (res) => {
    if (res.status() >= 400) problems.push(`HTTP ${res.status()}: ${res.url()}`);
  });

  await page.goto(url);
  await page.locator('#title:not([hidden]), #error:not([hidden])').first().waitFor({ timeout: TIMEOUT_MS });

  if (await page.locator('#error').isVisible()) {
    // The page's own i18n module gives the expected messages in its language.
    const allowed = await page.evaluate(async () => {
      const { t } = await import('./src/i18n.js');
      return [t('error.noWebgpu'), t('error.noAdapter')];
    });
    const shown = (await page.locator('#errorText').textContent()).trim();
    const noWebgpu = allowed.includes(shown);
    if (!noWebgpu) problems.push(`"Cannot start" screen with an unexpected reason: ${shown}`);
    // main.js logs the reason it shows, so that one console error is expected.
    consoleErrors = consoleErrors.filter((text) => !text.includes(shown));
    outcome = noWebgpu ? `no WebGPU here; the "Cannot start" screen shows: ${shown}` : 'the game could not start';
  } else {
    await page.locator('#title [data-action="start"]').click();
    await page.locator('#hud').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    await page.waitForTimeout(PLAY_MS);
    if (await page.locator('#error').isVisible()) problems.push('the "Cannot start" screen appeared during the run');
    if (!(await page.locator('#hud').isVisible())) problems.push('the HUD disappeared during the run');
    outcome = `WebGPU available; played ${PLAY_MS / 1000} s of a run`;
  }
} catch (err) {
  problems.push(`smoke test failed: ${err.message}`);
} finally {
  await browser.close();
  server.close();
}

problems.push(...consoleErrors.map((text) => `console error: ${text}`));
if (outcome) console.log(`smoke: ${outcome}`);
if (problems.length) {
  for (const problem of problems) console.error(`  ${problem}`);
  console.error(`smoke: failed with ${problems.length} problem(s)`);
  process.exitCode = 1;
} else {
  console.log('smoke: ok');
}
