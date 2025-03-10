use std::f32::INFINITY;

#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub start: f32,
    pub end: f32,
}

impl Interval {
    pub const EMPTY: Self = Interval::new(INFINITY, -INFINITY);
    pub const UNIVERSE: Self = Interval::new(-INFINITY, INFINITY);

    pub const fn new(start: f32, end: f32) -> Interval {
        Interval { start, end }
    }

    pub const fn combine(a: &Interval, b: &Interval) -> Interval {
        Interval {
            start: if a.start <= b.start { a.start } else { b.start },
            end: if a.end >= b.end { a.end } else { b.end },
        }
    }

    pub const fn size(&self) -> f32 {
        self.end - self.start
    }

    pub fn contains(&self, v: f32) -> bool {
        self.start <= v && v <= self.end
    }

    pub fn surrounds(&self, v: f32) -> bool {
        self.start < v && v < self.end
    }

    pub fn clamp(&self, v: f32) -> f32 {
        if v < self.start {
            self.start
        } else if v > self.end {
            self.end
        } else {
            v
        }
    }

    pub const fn expand(&mut self, delta: f32) {
        self.start -= delta / 2.0;
        self.end += delta / 2.0;
    }

    pub const fn expanded(&self, delta: f32) -> Interval {
        Interval::new(self.start - delta / 2.0, self.end + delta / 2.0)
    }
}

impl Default for Interval {
    fn default() -> Self {
        Interval::EMPTY
    }
}
