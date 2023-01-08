//! Message handler for making asynchronous IO call to RSS API.
use crate::{AppModel, AppMsg};
use my_weather::get_weather;
use relm4::{send, MessageHandler, Model, Sender};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{channel, Sender as TokioSender};

/// Message types for the AsyncHandler.
#[derive(Debug)]
pub enum AsyncHandlerMsg {
  /// Fetch weather data.
  Fetch,
}

/// An async handler for handling asynchronous IO requests
pub struct AsyncHandler {
  _rt: Runtime,
  sender: TokioSender<AsyncHandlerMsg>,
}

impl MessageHandler<AppModel> for AsyncHandler {
  type Msg = AsyncHandlerMsg;
  type Sender = TokioSender<AsyncHandlerMsg>;

  fn init(
    _parent_model: &AppModel,
    parent_sender: Sender<<AppModel as Model>::Msg>,
  ) -> Self {
    let (sender, mut rx) = channel::<AsyncHandlerMsg>(5);

    let rt = Builder::new_multi_thread()
      .worker_threads(2)
      .enable_io()
      .enable_time()
      .build()
      .expect("Tokio runtime");

    rt.spawn(async move {
      while let Some(msg) = rx.recv().await {
        let parent_sender = parent_sender.clone();
        tokio::spawn(async move {
          match msg {
            AsyncHandlerMsg::Fetch => {
              send!(parent_sender, AppMsg::Fetching);
              match get_weather().await {
                Ok(forecast) => {
                  // println!("received forecast {forecast:?}");
                  send!(parent_sender, AppMsg::Received(forecast))
                }
                Err(e) => {
                  // eprintln!("received error {e}");
                  send!(parent_sender, AppMsg::Error(e.to_string()))
                }
              }
            }
          }
        });
      }
    });

    // Make an initial fetch request on startup.
    if let Err(err) = sender.blocking_send(AsyncHandlerMsg::Fetch) {
      eprintln!("Failed to send fetch request {err}");
    }

    AsyncHandler { _rt: rt, sender }
  }

  fn send(&self, msg: Self::Msg) {
    if let Err(err) = self.sender.blocking_send(msg) {
      eprint!("Failed to send message {err}");
    }
  }

  fn sender(&self) -> Self::Sender {
    self.sender.clone()
  }
}
