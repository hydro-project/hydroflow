pub struct Counter {
    counter: usize,
}

impl Counter {
    pub fn new() -> Self {
        Counter { counter: 0 }
    }

    pub fn next(&mut self) -> usize {
        let ret = self.counter;
        self.counter += 1;
        ret
    }
}
