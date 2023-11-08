use super::*;

pub struct BattlescapesRunner {
    runners: Vec<RunnerHandle>,
    last_rebalance: Instant,

    new_battlescape_receiver: Receiver<Box<Battlescape>>,
}
impl BattlescapesRunner {
    /// Do not attempt rebalance if runner step duration is below this threshold.
    const REBALANCE_THRESHOLD_MIN_ABSOLUTE: f32 = TARGET_DT * 0.70;
    /// Do not attempt rebalance if runner step duration is below
    /// this threshold relative to the average runner step duration.
    const REBALANCE_THRESHOLD_MIN_RELATIVE: f32 = 1.3;
    /// Rebalance at most every this many seconds.
    const REBALANCE_INTERVAL: Duration = Duration::from_secs(10);

    pub fn new(num_runner: usize) -> (Self, Sender<Box<Battlescape>>) {
        let (new_battlescape_sender, new_battlescape_receiver) = channel();

        (
            Self {
                runners: (0..num_runner).map(|_| RunnerHandle::new()).collect(),
                last_rebalance: Instant::now(),
                new_battlescape_receiver,
            },
            new_battlescape_sender,
        )
    }

    pub fn step(&mut self, delta: f32) {
        // Step.
        for runner in self.runners.iter_mut() {
            runner.runner.delta = delta;
            runner.start();
        }
        for runner in self.runners.iter_mut() {
            runner.finish();
        }

        let mut battlescapes = Vec::new();

        // Take new battlescapes.
        while let Ok(battlescape) = self.new_battlescape_receiver.try_recv() {
            battlescapes.push(battlescape);
        }

        // Rebalance if needed.
        let threshold = if self.last_rebalance.elapsed() > Self::REBALANCE_INTERVAL {
            self.last_rebalance = Instant::now();

            let threshold = self.threshold();

            for runner in self.runners.iter_mut() {
                while runner.runner.average_step_duration > threshold
                    && runner.runner.battlescapes.len() > 1
                {
                    let battlescape = runner.runner.battlescapes.swap_remove(0);
                    runner.runner.average_step_duration -= battlescape.step_duration;
                    battlescapes.push(battlescape);
                }
            }

            Some(threshold)
        } else {
            None
        };

        if !battlescapes.is_empty() {
            let threshold = threshold.unwrap_or_else(|| self.threshold()) * 0.8;

            log::debug!(
                "Rebalancing runners with {} battlescapes. Threshold: {}",
                battlescapes.len(),
                threshold
            );

            'outer: for battlescape in battlescapes {
                let mut lowest = (0, f32::MAX);
                for (idx, runner) in self.runners.iter_mut().enumerate() {
                    if runner.runner.average_step_duration < threshold {
                        runner.runner.battlescapes.push(battlescape);
                        continue 'outer;
                    }
                    if runner.runner.average_step_duration < lowest.1 {
                        lowest = (idx, runner.runner.average_step_duration);
                    }
                }
                self.runners[lowest.0].runner.battlescapes.push(battlescape);
            }
        }
    }

    fn threshold(&self) -> f32 {
        let average_step_duration = self.runners.iter().fold(0.0f32, |acc, runner| {
            acc + runner.runner.average_step_duration
        }) / self.runners.len() as f32;

        (average_step_duration * Self::REBALANCE_THRESHOLD_MIN_RELATIVE)
            .max(Self::REBALANCE_THRESHOLD_MIN_ABSOLUTE)
    }
}

struct RunnerHandle {
    runner: RunnerInner,
    start_sender: SyncSender<RunnerInner>,
    finish_receiver: Receiver<RunnerInner>,
}
impl RunnerHandle {
    fn new() -> Self {
        let (start_sender, start_receiver) = sync_channel(1);
        let (finish_sender, finish_receiver) = sync_channel(1);

        std::thread::spawn(move || runner_loop(start_receiver, finish_sender));

        Self {
            runner: Default::default(),
            start_sender,
            finish_receiver,
        }
    }

    fn start(&mut self) {
        self.start_sender
            .send(std::mem::take(&mut self.runner))
            .unwrap();
    }

    fn finish(&mut self) {
        self.runner = self.finish_receiver.recv().unwrap();
    }
}

#[derive(Default)]
struct RunnerInner {
    battlescapes: Vec<Box<Battlescape>>,
    /// Moving average of the duration taken to step all battlescapes.
    average_step_duration: f32,
    /// Duration since last step.
    delta: f32,
}
impl RunnerInner {
    fn step(&mut self) {
        let start = Instant::now();
        let mut end = start;

        for battlescape in self.battlescapes.iter_mut() {
            battlescape.step(&mut end, self.delta);
        }

        let elapsed = (end - start).as_secs_f32();
        self.average_step_duration = self.average_step_duration * 0.98 + elapsed * 0.02;
    }
}

fn runner_loop(start_receiver: Receiver<RunnerInner>, finish_sender: SyncSender<RunnerInner>) {
    while let Ok(mut runner) = start_receiver.recv() {
        runner.step();
        finish_sender.send(runner).unwrap();
    }
}
