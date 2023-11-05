use super::*;
use battlescape::*;

pub struct SimulationRunnerHandle {
    handle: std::thread::JoinHandle<()>,
}
impl SimulationRunnerHandle {
    pub fn start() -> Self {
        let handle = std::thread::spawn(|| {
            SimulationRunner {}.run();
        });

        Self { handle }
    }
}

struct SimulationRunner {
    //
}
impl SimulationRunner {
    fn run(mut self) {
        let mut now = std::time::Instant::now();
        let mut sim_time = 0.0f64;
        let mut real_time = 0.0f64;
        loop {
            real_time += now.elapsed().as_secs_f64();
            now = std::time::Instant::now();

            let dif = sim_time - real_time;
            if dif < -DT as f64 * 4.0 {
                log::warn!("Simulation runner is lagging behind by {} seconds", -dif);
                real_time = sim_time + DT as f64 * 4.0;
            } else if dif > 0.001 {
                std::thread::sleep(std::time::Duration::from_secs_f64(dif));
            }

            self.step();
            sim_time += DT as f64;

            // if self.central_server_connection.disconnected {
            //     break;
            // }
        }
    }

    fn step(&mut self) {
        //
    }
}
