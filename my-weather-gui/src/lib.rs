mod error_dialog;
mod handlers;
mod parsers;
mod types;
mod widgets;

use crate::{error_dialog::DialogMsg, types::to_forecast};
use chrono::Local;
use error_dialog::ErrorDialogModel;
use handlers::AsyncHandler;
use my_weather::ForeCast;
use relm4::{
  factory::collections::FactoryVec, AppUpdate, Components, Model,
  RelmComponent, RelmMsgHandler, Sender,
};
use types::{ForeCastEntry, Temperature};
use widgets::AppWidgets;

pub use types::Celsius;

/// Application state.
pub struct AppModel<Unit>
where
  Temperature<Unit>: std::fmt::Display,
{
  forecast: FactoryVec<ForeCastEntry<Unit>>,
  fetching: bool,
  error: bool,
  status_message: String,
}

impl<Unit> Default for AppModel<Unit>
where
  Temperature<Unit>: std::fmt::Display,
{
  fn default() -> Self {
    Self {
      forecast: FactoryVec::new(),
      fetching: false,
      error: false,
      status_message: String::new(),
    }
  }
}

/// Application messages.
pub enum AppMsg {
  /// Fetching weather forecast.
  Fetching,
  /// Error has occurred.
  Error(String),
  /// Receive a weather forecast.
  Received(ForeCast),
  /// Clear weather forecasts entries.
  Clear,
}

impl<Unit> Model for AppModel<Unit>
where
  Temperature<Unit>: std::fmt::Display,
{
  type Msg = AppMsg;
  type Widgets = AppWidgets;
  type Components = AppComponents<Unit>;
}

impl<Unit> AppUpdate for AppModel<Unit>
where
  Temperature<Unit>: std::fmt::Display,
{
  fn update(
    &mut self,
    msg: Self::Msg,
    components: &Self::Components,
    _sender: Sender<Self::Msg>,
  ) -> bool {
    use AppMsg::*;

    match msg {
      Received(forecast) => {
        self.fetching = false;
        self.forecast.clear();
        for fc in to_forecast(forecast.entries()) {
          self.forecast.push(fc)
        }
        self.status_message =
          format!("Loaded weather at {}", Local::now().format("%v %r"));
      }
      Fetching => {
        self.fetching = true;
        self.error = false;
      }
      Clear => self.forecast.clear(),
      Error(error) => {
        self.fetching = false;
        self.error = true;
        if let Err(err) = components.error_dialog.send(DialogMsg::Open(error)) {
          eprintln!("Failed to send error to dialog component {err}");
        }
      }
    }
    true
  }
}

/// Background component for asynchronous IO requests.
// #[derive(relm4::Components)]
pub struct AppComponents<Unit>
where
  Temperature<Unit>: std::fmt::Display,
{
  async_handler: RelmMsgHandler<AsyncHandler, AppModel<Unit>>,
  error_dialog: RelmComponent<ErrorDialogModel, AppModel<Unit>>,
}

impl<Unit> Components<AppModel<Unit>> for AppComponents<Unit>
where
  Temperature<Unit>: std::fmt::Display,
{
  fn init_components(
    parent_model: &AppModel<Unit>,
    parent_sender: Sender<<AppModel<Unit> as Model>::Msg>,
  ) -> Self {
    AppComponents {
      async_handler: RelmMsgHandler::new(parent_model, parent_sender.clone()),
      error_dialog: RelmComponent::new(parent_model, parent_sender),
    }
  }
  fn connect_parent(
    &mut self,
    _parent_widgets: &<AppModel<Unit> as Model>::Widgets,
  ) {
  }
}
