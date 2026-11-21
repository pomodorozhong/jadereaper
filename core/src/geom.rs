//! Hit areas on the ground plane. Player skills and enemy attacks use the same shapes, and the warnings drawn
//! before an enemy attack show the same area that will hit.

/// The shape of a hit area, placed at an origin and facing a direction.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shape {
    /// A disc of radius `r` around the origin.
    Circle { r: f32 },
    /// A sector of radius `r`, `half` radians to each side of the direction.
    Arc { r: f32, half: f32 },
    /// A strip from the origin along the direction, `len` long and `half_w` to each side.
    Line { len: f32, half_w: f32 },
    /// The band between two radii around the origin.
    Ring { inner: f32, outer: f32 },
}

/// A placed hit area.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    /// Unit direction the area faces.
    pub dx: f32,
    pub dy: f32,
    pub shape: Shape,
}

impl Area {
    pub fn new(x: f32, y: f32, dir: (f32, f32), shape: Shape) -> Area {
        let l = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
        let (dx, dy) = if l > 1e-6 && l.is_finite() {
            (dir.0 / l, dir.1 / l)
        } else {
            (1.0, 0.0)
        };
        Area {
            x,
            y,
            dx,
            dy,
            shape,
        }
    }

    /// True when a round target at (tx, ty) with radius `tr` touches the area.
    pub fn hits(&self, tx: f32, ty: f32, tr: f32) -> bool {
        let (ox, oy) = (tx - self.x, ty - self.y);
        let d2 = ox * ox + oy * oy;
        match self.shape {
            Shape::Circle { r } => d2 < (r + tr) * (r + tr),
            Shape::Ring { inner, outer } => {
                let d = d2.sqrt();
                d > inner - tr && d < outer + tr
            }
            Shape::Arc { r, half } => {
                if d2 >= (r + tr) * (r + tr) {
                    return false;
                }
                let d = d2.sqrt();
                if d <= tr {
                    return true;
                }
                // The target's own size widens the angle it can be caught at.
                let cos = (ox * self.dx + oy * self.dy) / d;
                let ang = cos.clamp(-1.0, 1.0).acos();
                ang <= half + (tr / d).min(1.0).asin()
            }
            Shape::Line { len, half_w } => {
                let along = ox * self.dx + oy * self.dy;
                let side = (-ox * self.dy + oy * self.dx).abs();
                along > -tr && along < len + tr && side < half_w + tr
            }
        }
    }

    /// The farthest the area reaches from its origin.
    pub fn reach(&self) -> f32 {
        match self.shape {
            Shape::Circle { r } | Shape::Arc { r, .. } => r,
            Shape::Line { len, .. } => len,
            Shape::Ring { outer, .. } => outer,
        }
    }
}

pub fn dist(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    ((ax - bx) * (ax - bx) + (ay - by) * (ay - by)).sqrt()
}

/// Unit vector from a to b, or (1, 0) when they are on the same spot.
pub fn dir_to(ax: f32, ay: f32, bx: f32, by: f32) -> (f32, f32) {
    let (dx, dy) = (bx - ax, by - ay);
    let d = (dx * dx + dy * dy).sqrt();
    if d > 1e-4 {
        (dx / d, dy / d)
    } else {
        (1.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        let c = Area::new(0.0, 0.0, (1.0, 0.0), Shape::Circle { r: 10.0 });
        assert!(c.hits(14.0, 0.0, 5.0));
        assert!(!c.hits(16.0, 0.0, 5.0));

        let a = Area::new(
            0.0,
            0.0,
            (1.0, 0.0),
            Shape::Arc {
                r: 100.0,
                half: 0.5,
            },
        );
        assert!(a.hits(50.0, 0.0, 1.0));
        assert!(!a.hits(-50.0, 0.0, 1.0));
        assert!(!a.hits(0.0, 50.0, 1.0));
        assert!(
            a.hits(0.0, 2.0, 5.0),
            "a target overlapping the origin is hit"
        );

        let l = Area::new(
            0.0,
            0.0,
            (0.0, 1.0),
            Shape::Line {
                len: 100.0,
                half_w: 10.0,
            },
        );
        assert!(l.hits(5.0, 90.0, 1.0));
        assert!(!l.hits(20.0, 50.0, 1.0));
        assert!(!l.hits(0.0, 120.0, 1.0));

        let r = Area::new(
            0.0,
            0.0,
            (1.0, 0.0),
            Shape::Ring {
                inner: 50.0,
                outer: 60.0,
            },
        );
        assert!(r.hits(55.0, 0.0, 1.0));
        assert!(!r.hits(20.0, 0.0, 1.0));
    }
}
