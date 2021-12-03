pub struct Time {
    pub s: usize,
    pub us: usize,
}

impl Time {
    pub fn new() -> Self {
        Self { s: 0, us: 0 }
    }
}
