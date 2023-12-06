use super::*;

pub enum BattlescapeRunnerRequest {
    Step,
}

pub enum BattlescapeRunnerResponse {
    // TODO
}

pub struct BattlescapeRunnerHandle {
    pub request_sender: Sender<BattlescapeRunnerRequest>,
    pub response_receiver: Receiver<BattlescapeRunnerResponse>,
}
impl BattlescapeRunnerHandle {
    pub fn start(battlescape: Box<Battlescape>) -> Self {
        let (request_sender, request_receiver) = unbounded();
        let (response_sender, response_receiver) = unbounded();

        std::thread::spawn(move || {
            BattlescapeRunner {
                battlescape,
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
    battlescape: Box<Battlescape>,
    request_receiver: Receiver<BattlescapeRunnerRequest>,
    response_sender: Sender<BattlescapeRunnerResponse>,
}
impl BattlescapeRunner {
    fn run(mut self) {
        for request in self.request_receiver.iter() {
            match request {
                BattlescapeRunnerRequest::Step => self.battlescape.step(),
            }
        }
    }
}
