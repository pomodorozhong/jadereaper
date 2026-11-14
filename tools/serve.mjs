// A small static file server for local play and the smoke test, with no
// dependencies. WebGPU needs a secure origin, which http://localhost is.
//
//     node tools/serve.mjs [port]   (default 5173; `npm start` runs it)

import { createReadStream, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { dirname, extname, join, normalize, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');

// Only what the Pages site publishes is served, so a file the game loads but
// the deploy leaves out fails here too. Must match the copy step in
// .github/workflows/pages.yml. Entries ending in '/' are folders.
const PUBLISHED = ['index.html', 'style.css', 'src/', 'wasm/', 'assets/'];

const isPublished = (rel) => PUBLISHED.some((p) => (p.endsWith('/') ? rel.startsWith(p) : rel === p));

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.wgsl': 'text/plain; charset=utf-8',
  '.wasm': 'application/wasm',
  '.woff2': 'font/woff2',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.svg': 'image/svg+xml',
  '.txt': 'text/plain; charset=utf-8',
};

/** Start serving the repository root on 127.0.0.1. Resolves to the server once it listens. */
export function serve(port = 5173) {
  const server = createServer((req, res) => {
    let path;
    try {
      path = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    } catch {
      res.writeHead(400).end();
      return;
    }
    if (path.endsWith('/')) path += 'index.html';
    const file = normalize(join(ROOT, path));
    let stat;
    try {
      stat = statSync(file);
    } catch {
      stat = null;
    }
    const inside = file.startsWith(ROOT + sep);
    const rel = inside ? file.slice(ROOT.length + 1).split(sep).join('/') : '';
    if (!inside || !isPublished(rel) || !stat || !stat.isFile()) {
      res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' }).end('Not found');
      return;
    }
    res.writeHead(200, {
      'Content-Type': TYPES[extname(file)] || 'application/octet-stream',
      'Content-Length': stat.size,
      'Cache-Control': 'no-cache',
    });
    createReadStream(file).pipe(res);
  });
  return new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', () => resolve(server));
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const port = Number(process.argv[2] || 5173);
  await serve(port);
  console.log(`Serving on http://localhost:${port}`);
}
