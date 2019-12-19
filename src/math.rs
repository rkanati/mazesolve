

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct V2 {
    pub x: i32,
    pub y: i32
}

impl V2 {
    pub const fn new(x: i32, y: i32) -> V2 {
        V2 { x, y }
    }
}

impl std::ops::Add for V2 {
    type Output = V2;
    fn add(self, rhs: V2) -> V2 {
        V2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for V2 {
    type Output = V2;
    fn sub(self, rhs: V2) -> V2 {
        V2 { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

// XXX rects are half-open
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rect {
    pub mins: V2,
    pub maxs: V2,
}

impl Rect {
    pub fn new(a: V2, b: V2) -> Rect {
        let xmin = a.x.min(b.x);
        let ymin = a.y.min(b.y);
        let xmax = a.x.max(b.x);
        let ymax = a.y.max(b.y);
        let mins = V2::new(xmin, ymin);
        let maxs = V2::new(xmax, ymax);
        Rect { mins, maxs }
    }

    pub fn new_unchecked(mins: V2, maxs: V2) -> Rect {
        let r = Rect { mins, maxs };
        r.width();
        r.height();
        r
    }

    pub fn width(&self) -> i32 {
        let w = self.maxs.x - self.mins.x;
        debug_assert!(w >= 0);
        w
    }

    pub fn height(&self) -> i32 {
        let h =self.maxs.y - self.mins.y;
        debug_assert!(h >= 0);
        h
    }

  //fn dims(&self) -> V2 {
  //    V2::new(self.width(), self.height())
  //}

    pub fn intersect(&self, other: Rect) -> Rect {
        let xmin = self.mins.x.max(other.mins.x);
        let ymin = self.mins.y.max(other.mins.y);
        let xmax = self.maxs.x.min(other.maxs.x);
        let ymax = self.maxs.y.min(other.maxs.y);
        let mins = V2::new(xmin, ymin);
        let maxs = V2::new(xmax, ymax);
        Rect { mins, maxs }
    }

    pub fn contains(&self, p: V2) -> bool {
        p.x >= self.mins.x &&
        p.y >= self.mins.y &&
        p.x <  self.maxs.x &&
        p.y <  self.maxs.y
    }
}

