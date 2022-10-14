use crossbeam::channel::{bounded, Receiver, Sender};
use std::thread::spawn;

use crate::battlescape::battlescape_inner::BattlescapeInner;

pub trait Runnable {
    fn run(&mut self);
}
impl Runnable for BattlescapeInner {
    fn run(&mut self) {
        self.step();
    }
}

/// Run something on a separate thread.
pub struct RunnerHandle<T: Runnable + Send> {
    runner_sender: Sender<Box<T>>,
    runner_receiver: Receiver<Box<T>>,
    pub t: Option<Box<T>>,
}
impl<T: Runnable + Send + 'static> RunnerHandle<T> {
    pub fn new(t: T) -> Self {
        let (runner_sender, _runner_receiver) = bounded(1);
        let (_runner_sender, runner_receiver) = bounded(1);

        spawn(move || Self::runner(_runner_receiver, _runner_sender));

        Self {
            runner_sender,
            runner_receiver,
            t: Some(Box::new(t)),
        }
    }

    /// Handle communication with the runner thread.
    ///
    /// Return `T` if it not being updated.
    pub fn update(&mut self) -> Option<&mut T> {
        // Try to fetch.
        match self.runner_receiver.try_recv() {
            Ok(t) => {
                self.t = Some(t);
                self.t.as_deref_mut()
            }
            Err(crossbeam::channel::TryRecvError::Empty) => {
                // Still updating or we already have it.
                self.t.as_deref_mut()
            }
            Err(crossbeam::channel::TryRecvError::Disconnected) => {
                // Runner has crashed.
                log::error!("Runner disconnected.");
                panic!()
            }
        }
    }

    /// Ask to run on another thread.
    ///
    /// **`T` should be on this thread.**
    ///
    /// You will be notified when it comes back when calling `update()`.
    pub fn run(&mut self) {
        if let Some(t) = self.t.take() {
            self.runner_sender.send(t).unwrap();
        } else {
            log::error!(
                "Asked to step the battlescape when it was not present on the main thread. Ignoring..."
            );
        }
    }

    /// Run and send back to the channel.
    fn runner(runner_receiver: Receiver<Box<T>>, runner_sender: Sender<Box<T>>) {
        while let Ok(mut t) = runner_receiver.recv() {
            t.run();
            runner_sender.send(t).unwrap()
        }
    }
}


