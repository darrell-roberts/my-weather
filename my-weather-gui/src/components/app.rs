use super::{
  error_dialog::{DialogMsg, ErrorDialogModel},
  forecast_factory::ForecastEntryAndTempUnit,
  header_menu::{HeaderModel, HeaderMsg},
  refresh_worker::{RefreshWorker, RefreshWorkerShutdown},
};
use crate::TempUnit;
use chrono::Local;
use my_weather::{get_weather, types::to_forecast, ApiError, ForeCast};
use relm4::{
  factory::FactoryVecDeque, gtk::prelude::*, Component, ComponentController, ComponentParts,
  Controller, RelmWidgetExt,
};

/// Application state.
pub struct AppModel {
  forecast: FactoryVecDeque<ForecastEntryAndTempUnit>,
  fetching: bool,
  status_message: String,
  status_dialog: Controller<ErrorDialogModel>,
  header: Controller<HeaderModel>,
  refresh_timer: Controller<RefreshWorker>,
}

#[derive(Debug)]
/// Application messages.
pub enum AppMsg {
  /// Error has occurred.
  Error(String),
  /// Clear weather forecasts entries.
  ChangeUnit(TempUnit),
  /// Request fetch new forecast data
  Fetch,
}

#[derive(Debug)]
pub enum FetchWeather {
  Fetched(Result<ForeCast, ApiError>),
}

#[relm4::component(pub)]
impl Component for AppModel {
  type Init = Result<ForeCast, ApiError>;
  type Input = AppMsg;
  type Output = ();
  type CommandOutput = FetchWeather;

  view! {
      gtk::Window {
        set_title: Some("My Weather"),
        set_titlebar: Some(header),
        set_resizable: true,
        set_default_size: (400, if cfg!(target_os = "macos") { 680 } else { 750 }),

        #[name = "window"]
        gtk::Box {
          set_orientation: gtk::Orientation::Vertical,

          // Content area.
          #[name = "scroll"]
          gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_width: true,
            set_propagate_natural_height: true,
            #[name = "container"]
            gtk::Box {
              set_orientation: gtk::Orientation::Vertical,
              set_margin_all: 5,
              set_spacing: 5,
              set_hexpand: true,
              set_vexpand: true,

              // Forecast data.
              #[local_ref]
              forecast_factory -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 5,
                set_spacing: 5,
              },
          },
        },

        // Refresh button.
        #[name = "reload"]
        gtk::Button {
          set_label: "Refresh",
          #[watch]
          set_sensitive: !model.fetching,
          set_halign: gtk::Align::Center,
          set_css_classes: &["refresh"],
          connect_clicked[sender] => move |_| {
            sender.input(AppMsg::Fetch)
          }
        },

        // Status bar.
        #[name = "status"]
        gtk::Statusbar {
          set_halign: gtk::Align::Fill,
        },
      },
    }
  }

  fn init(
    forecast: Self::Init,
    root: &Self::Root,
    sender: relm4::ComponentSender<Self>,
  ) -> relm4::ComponentParts<Self> {
    let mut model = AppModel {
      forecast: FactoryVecDeque::new(gtk::Box::default(), sender.input_sender()),
      fetching: false,
      status_dialog: ErrorDialogModel::builder().launch(()).detach(),
      status_message: String::new(),
      header: HeaderModel::builder()
        .launch(())
        .forward(sender.input_sender(), |msg| match msg {
          HeaderMsg::ChangeUnit(unit) => AppMsg::ChangeUnit(unit),
        }),
      refresh_timer: RefreshWorker::builder()
        .launch(())
        .forward(sender.input_sender(), |_| AppMsg::Fetch),
    };
    model.status_dialog.widget().set_transient_for(Some(root));
    model.handle_api_result(forecast);
    let forecast_factory = model.forecast.widget();
    let header = model.header.widget();
    let widgets = view_output!();

    ComponentParts { model, widgets }
  }

  fn update_cmd(
    &mut self,
    message: Self::CommandOutput,
    _sender: relm4::ComponentSender<Self>,
    _root: &Self::Root,
  ) {
    match message {
      FetchWeather::Fetched(result) => {
        self.fetching = false;
        self.handle_api_result(result);
      }
    }
  }

  fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>, _: &Self::Root) {
    use AppMsg::*;

    match message {
      Error(error) => {
        self.fetching = false;
        self.status_dialog.emit(DialogMsg::Open(error));
      }
      ChangeUnit(unit) => {
        self.forecast.guard().iter_mut().for_each(|fc| fc.1 = unit);
      }
      Fetch => {
        self.fetching = true;
        sender.oneshot_command(async { FetchWeather::Fetched(get_weather().await) });
        // sender.oneshot_command(async {
        //   FetchWeather::Fetched(Err(ApiError::TestError("blah".into())))
        // });
      }
    }
  }

  fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
    self.refresh_timer.emit(RefreshWorkerShutdown);
  }

  fn pre_view() {
    status.pop(STATUS_CONTEXT_ID);
    status.push(
      STATUS_CONTEXT_ID,
      if model.fetching {
        "Fetching forecast..."
      } else {
        &model.status_message
      },
    );
  }
}

const STATUS_CONTEXT_ID: u32 = 1;

impl AppModel {
  fn handle_api_result(&mut self, result: Result<ForeCast, ApiError>) {
    match result {
      Ok(forecast) => {
        self.forecast.guard().clear();
        for fc in to_forecast(forecast.entries()) {
          self.forecast.guard().push_back((fc, TempUnit::Celsius));
        }
        self.status_message = format!("Loaded weather at {}", Local::now().format("%v %r"));
      }
      Err(err) => self.status_dialog.emit(DialogMsg::Open(format!("{err}"))),
    }
  }
}
