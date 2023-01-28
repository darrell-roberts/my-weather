mod error_dialog;
mod handlers;
mod header_menu;
mod widgets;

use crate::error_dialog::DialogMsg;
use chrono::Local;
use error_dialog::ErrorDialogModel;
use handlers::AsyncHandler;
use header_menu::HeaderModel;
use my_weather::{
  types::{to_forecast, ForecastEntry},
  ForeCast,
};
use relm4::{
  factory::collections::FactoryVec, AppUpdate, Model, RelmComponent, RelmMsgHandler, Sender,
};
use widgets::AppWidgets;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TempUnit {
  Fahrenheit,
  #[default]
  Celsius,
}

#[derive(Debug)]
struct ForecastEntryAndTempUnit(ForecastEntry, TempUnit);

/// Application state.
pub struct AppModel {
  forecast: FactoryVec<ForecastEntryAndTempUnit>,
  fetching: bool,
  error: bool,
  status_message: String,
}

impl Default for AppModel {
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
  /// Change the temperature unit.
  ChangeUnit(TempUnit),
}

impl Model for AppModel {
  type Msg = AppMsg;
  type Widgets = AppWidgets;
  type Components = AppComponents;
}

impl AppUpdate for AppModel {
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
          self
            .forecast
            .push(ForecastEntryAndTempUnit(fc, TempUnit::Celsius));
        }
        self.status_message = format!("Loaded weather at {}", Local::now().format("%v %r"));
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
      ChangeUnit(unit) => {
        let mut fs = vec![];

        while let Some(mut fc) = self.forecast.pop() {
          fc.1 = unit;
          fs.push(fc);
        }

        for fc in fs.into_iter().rev() {
          self.forecast.push(fc);
        }
      }
    }
    true
  }
}

/// Background component for asynchronous IO requests.
#[derive(relm4::Components)]
pub struct AppComponents {
  async_handler: RelmMsgHandler<AsyncHandler, AppModel>,
  error_dialog: RelmComponent<ErrorDialogModel, AppModel>,
  header: RelmComponent<HeaderModel, AppModel>,
}
