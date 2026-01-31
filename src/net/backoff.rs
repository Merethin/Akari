use std::time::Duration;

pub struct ExponentialBackoff<'a> {
    index: usize,
    delays: &'a [u64],
}

impl ExponentialBackoff<'_> {
    pub fn new<'a>(
        delays: &'a [u64],
    ) -> ExponentialBackoff<'a> {
        assert!(!delays.is_empty(), "ExponentialBackoff must not be initialized with an empty array");
        ExponentialBackoff { index: 0, delays }
    }

    pub async fn wait(&mut self) {
        tokio::time::sleep(Duration::from_secs(self.delay())).await;
        self.index = self.index.saturating_add(1);
    }

    pub fn delay(&self) -> u64 {
        if self.index >= self.delays.len() {
            return *self.delays.last().expect("Delays should not be empty (verified by assertion)");
        }

        self.delays[self.index]
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}