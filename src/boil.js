// The line boil shared by the game (src/characters.js) and the Lightbox (tools/lightbox.html), per decision 0004,
// so the Lightbox's boil preview always matches the game.

/** Small seeded random number generator (mulberry32); returns a function giving numbers in [0, 1). */
export function rng(seed) {
	let a = seed >>> 0;
	return () => {
		a = (a + 0x6d2b79f5) >>> 0;
		let t = a;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

/**
 * The seed for one line of a part in one boil pose.
 * @param {number} kind 1 paper fill, 2 solid ink, 3 stroke
 * @param {number} i the line's index within its kind
 * @param {number} pose boil pose
 * @param {number} pi the part's index in its character, so parts boil apart
 */
export const boilSeed = (kind, i, pose, pi) =>
	(kind * 104729 + i * 31 + pose * 15485863 + pi * 7919) | 0;

/** Wobble a line's points with two seeded waves along its length, plus a small seeded offset. */
export function jitter(pts, closed, seed, amp) {
	if (!amp) return pts;
	const r = rng(seed),
		len = [0];
	for (let i = 1; i < pts.length; i++)
		len.push(
			len[i - 1] +
				Math.hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1]),
		);
	const T = len[len.length - 1] || 1,
		ox = (r() - 0.5) * amp,
		oy = (r() - 0.5) * amp,
		waves = [];
	for (let k = 0; k < 2; k++) {
		const lam = 20 + r() * 50;
		waves.push({
			fr: closed
				? (2 * Math.PI * Math.max(1, Math.round(T / lam))) / T
				: (2 * Math.PI) / lam,
			px: r() * 6.283,
			py: r() * 6.283,
			w: k ? 0.35 : 0.65,
		});
	}
	return pts.map(([x, y], i) => {
		let dx = 0,
			dy = 0;
		for (const v of waves) {
			dx += v.w * Math.sin(len[i] * v.fr + v.px);
			dy += v.w * Math.sin(len[i] * v.fr + v.py);
		}
		return [x + ox + dx * amp * 0.8, y + oy + dy * amp * 0.8];
	});
}

/**
 * A smooth curve through the points (uniform Catmull-Rom as cubic beziers). Two points give a straight line.
 * @returns {{start: number[], line?: number[], segs: number[][]}} each seg is [c1x, c1y, c2x, c2y, x, y]
 */
export function curveThrough(p, closed) {
	if (p.length === 2) return { start: p[0], line: p[1], segs: [] };
	let e;
	if (closed) {
		p = p.slice(0, -1);
		e = [p[p.length - 1], ...p, p[0], p[1]];
	} else e = [p[0], ...p, p[p.length - 1]];
	const segs = [];
	for (let i = 1; i < e.length - 2; i++) {
		const [a, b, c, q] = [e[i - 1], e[i], e[i + 1], e[i + 2]];
		segs.push([
			b[0] + (c[0] - a[0]) / 6,
			b[1] + (c[1] - a[1]) / 6,
			c[0] - (q[0] - b[0]) / 6,
			c[1] - (q[1] - b[1]) / 6,
			c[0],
			c[1],
		]);
	}
	return { start: e[1], segs };
}
