pub struct TimingBuffer {
    buffer: Vec<f64>,
    size: usize,
}

impl TimingBuffer {
    pub fn new(size: usize) -> TimingBuffer {
        TimingBuffer {
            buffer: Vec::with_capacity(size),
            size,
        }
    }

    pub fn avg(&self) -> f64 {
        let avg = 1.0 / (self.buffer.iter().sum::<f64>() / self.buffer.len() as f64);
        avg
    }

    pub fn add_time(&mut self, timing: f64) {
        if self.buffer.is_empty() {
            for _i in 1..self.size {
                self.buffer.push(timing);
            }
        } else {
            self.buffer.remove(0);
            self.buffer.push(timing);
        }
    }
}
