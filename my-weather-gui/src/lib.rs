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
    factory::collections::FactoryVec, AppUpdate, Model, RelmComponent, RelmMsgHandler, Sender,
};
use types::ForeCastEntry;
use widgets::AppWidgets;

/// Application state.
pub struct AppModel {
    forecast: FactoryVec<ForeCastEntry>,
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
                    self.forecast.push(fc)
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
        }
        true
    }
}

/// Background component for asynchronous IO requests.
#[derive(relm4::Components)]
pub struct AppComponents {
    async_handler: RelmMsgHandler<AsyncHandler, AppModel>,
    error_dialog: RelmComponent<ErrorDialogModel, AppModel>,
}
