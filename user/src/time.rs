use core::ops::Sub;
#[derive(Debug)]
pub struct Time {
    pub s: usize,
    pub us: usize,
}

impl Time {
    pub fn new() -> Self {
        Self { s: 0, us: 0 }
    }
}

impl Sub for Time {
    type Output = usize;
    fn sub(self, other: Time) -> usize {
        // println!("{:?}",self);
        // println!("{:?}",other);
        let mut s = 0usize;
        let mut us = 0usize;
        if self.s > other.s {
            s = self.s - other.s;
        }
        if self.us > other.us {
            us = self.us - other.us;
        } else if s > 0 {
            us = self.us - other.us + 1000;
        }
        if s > 0 {
            s * 1000 + us / 1000
        } else {
            us / 1000
        }
    }
}

impl From<Time> for usize {
    fn from(t: Time) -> Self {
        t.s * 1000 + t.us / 1000
    }
}
