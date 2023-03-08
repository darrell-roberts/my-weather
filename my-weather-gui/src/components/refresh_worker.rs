use relm4::{ComponentSender, Worker};
use std::{
  sync::atomic::{AtomicBool, Ordering},
  thread::{self, JoinHandle},
  time::Duration,
};

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct RefreshOutput;

#[derive(Debug)]
pub struct RefreshWorkerShutdown;

pub struct RefreshWorker(Option<JoinHandle<()>>);

impl Worker for RefreshWorker {
  type Init = ();
  type Input = RefreshWorkerShutdown;
  type Output = RefreshOutput;

  fn init(_init: Self::Init, sender: relm4::ComponentSender<Self>) -> Self {
    let handle = Some(start_timer(sender));
    Self(handle)
  }

  fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {
    RUNNING.store(false, Ordering::Relaxed);
    if let Some(handle) = self.0.take() {
      handle.join().unwrap();
    }
  }
}

fn start_timer(sender: ComponentSender<RefreshWorker>) -> JoinHandle<()> {
  RUNNING.store(true, Ordering::Relaxed);
  thread::spawn(move || {
    while RUNNING.load(Ordering::Relaxed) {
      thread::sleep(Duration::from_secs(10));
      sender.output(RefreshOutput).unwrap();
    }
  })
}
