//! Main application window widgets.
use crate::{
  handlers::AsyncHandlerMsg,
  types::{DayNight, ForecastEntry, ForecastWithEntry, Temperature},
  AppModel, AppMsg, ForecastEntryAndTempUnit, TempUnit,
};
use relm4::{
  factory::{collections::FactoryVec, FactoryPrototype},
  gtk::prelude::*,
  Sender, WidgetPlus, Widgets,
};

#[relm4_macros::widget(pub)]
impl Widgets<AppModel, ()> for AppWidgets {
  view! {
    gtk::ApplicationWindow {
      set_title: Some("My Weather"),
      set_titlebar: Some(components.header.root_widget()),
      set_default_width: 300,
      set_default_height: 100,

      set_child = scroll = Some(&gtk::ScrolledWindow) {
        set_hscrollbar_policy: gtk::PolicyType::Never,
        set_propagate_natural_width: true,
        set_propagate_natural_height: true,
        set_min_content_height: if cfg!(target_os = "macos") { 625 } else { 670 },
        set_child = container = Some(&gtk::Box) {
          set_orientation: gtk::Orientation::Vertical,
          set_margin_all: 5,
          set_spacing: 5,

          // Forecast data.
          append = weather = &gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_visible: watch! { !model.fetching },
            factory!(model.forecast)
          },

          // Fetching container.
          append = spinner_box = &gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_margin_all: 5,
            set_spacing: 5,
            set_visible: watch! { model.fetching },
            set_hexpand: true,
            set_vexpand: true,
            append = &gtk::Label {
              set_label: "Fetching weather..."
            },
            append = spinner = &gtk::Spinner {
            }
          },

          // Reload button.
          append = reload = &gtk::Button {
            set_label: "Refresh",
            set_sensitive: watch! { !model.fetching },
            set_halign: gtk::Align::Center,
            set_css_classes: &["refresh"],
            connect_clicked[sender = components.async_handler.sender()] => move |_| {
              sender.blocking_send(AsyncHandlerMsg::Fetch).expect("Receiver dropped");
            }
          },

          append = status = &gtk::Statusbar {
            set_halign: gtk::Align::Fill,
          },
        }
      }
    }
  }

  fn pre_view() {
    let ctx_id = status.context_id("state");
    status.push(
      ctx_id,
      if model.fetching {
        "Loading weather..."
      } else {
        &model.status_message
      },
    );

    if model.fetching {
      spinner.start();
    } else {
      spinner.stop();
    }
  }
}

/// Widgets used to for each [ForeCastEntry].
#[derive(Debug, Default)]
pub struct FactoryWidgets {
  container: gtk::Box,
}

impl FactoryPrototype for ForecastEntryAndTempUnit {
  type Factory = FactoryVec<Self>;
  type Widgets = FactoryWidgets;
  type Root = gtk::Box;
  type View = gtk::Box;
  type Msg = AppMsg;

  fn init_view(
    &self,
    _key: &<Self::Factory as relm4::factory::Factory<Self, Self::View>>::Key,
    _sender: Sender<Self::Msg>,
  ) -> Self::Widgets {
    let container = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(5)
      .build();

    let mut row_container = gtk::Box::builder()
      .css_name("item")
      .orientation(gtk::Orientation::Vertical)
      .spacing(5);

    match self.0 {
      ForecastEntry::Current(_) => {
        row_container = row_container.css_classes(vec!["current".into()]);
      }
      ForecastEntry::Warning(_) => {
        row_container = row_container.css_classes(vec!["warning".into()]);
      }
      _ => (),
    }

    let row_container = row_container
      .orientation(gtk::Orientation::Vertical)
      .build();
    self.init_forecast(&row_container);
    container.append(&row_container);
    let separator = gtk::Separator::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Fill)
      .hexpand(true)
      .build();

    container.append(&separator);

    FactoryWidgets { container }
  }

  fn position(
    &self,
    _key: &<Self::Factory as relm4::factory::Factory<Self, Self::View>>::Key,
  ) -> <Self::View as relm4::factory::FactoryView<Self::Root>>::Position {
  }

  fn view(
    &self,
    key: &<Self::Factory as relm4::factory::Factory<Self, Self::View>>::Key,
    _widgets: &Self::Widgets,
  ) {
    println!("Updating key {key}");
  }

  fn root_widget(widgets: &Self::Widgets) -> &Self::Root {
    &widgets.container
  }
}

impl ForecastEntryAndTempUnit {
  /// Build widgets inside a forecast container.
  fn init_forecast(&self, row_container: &gtk::Box) {
    match &self.0 {
      ForecastEntry::Future { forecast, .. } => {
        self.init_future_forecast(forecast, row_container);
      }
      ForecastEntry::Current(forecast) => {
        self.init_current_forecast(forecast, row_container);
      }
      ForecastEntry::Warning(entry) => {
        row_container.append(
          &gtk::Label::builder()
            .halign(gtk::Align::Center)
            .tooltip_markup(&self.0.summary())
            .label(&entry.title)
            .build(),
        );
      }
    }
  }

  /// Build widgets for a current forecast.
  fn init_current_forecast(
    &self,
    forecast: &crate::types::CurrentForecastWithEntry,
    row_container: &gtk::Box,
  ) {
    let day_of_week_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .build();
    day_of_week_container.append(
      &gtk::Label::builder()
        .label("Current")
        .css_classes(vec!["dayofweek".into()])
        .tooltip_markup(&self.0.summary())
        .build(),
    );
    let info_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .spacing(5)
      .build();
    let temp_string = if self.1 == TempUnit::Celsius {
      format!("{}", forecast.current.celsius)
    } else {
      format!("{}", forecast.current.fahrenheit)
    };
    info_container.append(
      &gtk::Label::builder()
        .css_name("temperature")
        .css_classes(vec!["current".into()])
        .label(&temp_string)
        .build(),
    );
    info_container.append(
      &gtk::Label::builder()
        .css_name("description")
        .label(&forecast.current.description)
        .build(),
    );
    row_container.append(&day_of_week_container);
    row_container.append(&info_container);
  }

  /// Build widgets for a future forecast.
  fn init_future_forecast(&self, forecast: &[ForecastWithEntry], row_container: &gtk::Box) {
    let day_of_week_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .build();
    row_container.append(&day_of_week_container);
    let day_night_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .spacing(5)
      .build();
    if let Some(day) = forecast
      .iter()
      .next()
      .map(|fc| fc.forecast.day_of_week.as_str())
    {
      day_of_week_container.append(
        &gtk::Label::builder()
          .label(day)
          .css_classes(vec!["dayofweek".into()])
          .tooltip_markup(&self.0.summary())
          .build(),
      );
    }

    if forecast.len() == 1 {
      if forecast[0].forecast.day == DayNight::Day {
        day_night_container.set_halign(gtk::Align::Start);
      } else {
        day_night_container.set_halign(gtk::Align::End);
      }
    }

    for ForecastWithEntry { forecast, .. } in forecast {
      let temp_string = if self.1 == TempUnit::Celsius {
        format!("{}", &forecast.celsius)
      } else {
        format!("{}", &forecast.fahrenheit)
      };
      let high_low_label = gtk::Label::builder()
        .css_name("temperature")
        .css_classes(vec![match &forecast.celsius {
          Temperature::High(..) => "high".into(),
          Temperature::Low(..) => "low".into(),
          Temperature::Current(..) => "current".into(),
        }])
        .label(&temp_string);

      let mut day_night_label = gtk::Label::builder()
        .css_name("description")
        .label(&forecast.description);
      match forecast.day {
        DayNight::Day => {
          day_night_label = day_night_label
            .css_classes(vec!["day".into()])
            .halign(gtk::Align::Start)
        }
        DayNight::Night => day_night_label = day_night_label.css_classes(vec!["night".into()]),
      }

      day_night_container.append(&high_low_label.build());
      day_night_container.append(&day_night_label.build());
    }
    row_container.append(&day_night_container);
  }
}
