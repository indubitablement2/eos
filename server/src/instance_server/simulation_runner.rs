use super::*;
use battlescape::*;

pub struct BattlescapeHandle {
    handle: std::thread::JoinHandle<()>,
    pub clients: Mutex<AHashSet<ClientId>>,
}
impl BattlescapeHandle {
    pub fn start() -> Self {
        let handle = std::thread::spawn(|| {
            BattlescapeRunner {}.run();
        });

        Self {
            handle,
            clients: Default::default(),
        }
    }
}

struct BattlescapeRunner {
    //
}
impl BattlescapeRunner {
    fn run(mut self) {
        // let mut previous_step = Instant::now();
        // loop {
        //     if let Some(remaining) = TARGET_DT_DURATION.checked_sub(previous_step.elapsed()) {
        //         std::thread::sleep(remaining);
        //     }

        //     let now = Instant::now();
        //     let delta = (now - previous_step).as_secs_f32().min(TARGET_DT * 2.0);
        //     previous_step = now;
        //     runner.step(delta);
        // }

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
