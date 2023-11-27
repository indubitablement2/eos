use super::*;
use battlescape::*;

pub enum BattlescapeRunnerRequest {
    // TODO
}

pub enum BattlescapeRunnerResponse {
    // TODO
}

pub struct BattlescapeRunnerHandle {
    pub request_sender: Sender<BattlescapeRunnerRequest>,
    pub response_receiver: Receiver<BattlescapeRunnerResponse>,
}
impl BattlescapeRunnerHandle {
    pub fn start() -> Self {
        let (request_sender, request_receiver) = channel();
        let (response_sender, response_receiver) = channel();

        std::thread::spawn(move || {
            BattlescapeRunner {
                request_receiver,
                response_sender,
            }
            .run();
        });

        Self {
            request_sender,
            response_receiver,
        }
    }
}

struct BattlescapeRunner {
    // battlescape: Battlescape,
    request_receiver: Receiver<BattlescapeRunnerRequest>,
    response_sender: Sender<BattlescapeRunnerResponse>,
}
impl BattlescapeRunner {
    fn run(mut self) {
        let mut interval = interval::Interval::new(DT_MS);
        loop {
            interval.step();

            for request in self.request_receiver.try_iter() {
                match request {
                    // TODO
                }
            }

            self.step();
        }
    }

    fn step(&mut self) {
        //
    }
}
