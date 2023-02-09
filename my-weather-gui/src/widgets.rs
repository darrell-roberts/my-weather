//! Main application window widgets.
use crate::{handlers::AsyncHandlerMsg, AppModel, AppMsg, ForecastEntryAndTempUnit, TempUnit};
use gtk::pango::EllipsizeMode;
use my_weather::types::{
  CurrentForecastWithEntry, DayNight, Forecast, ForecastEntry, ForecastWithEntry, Temperature,
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
        set_resizable: true,
        set_default_size: args!(400, if cfg!(target_os = "macos") { 680 } else { 750 }),

        set_child = window = Some(&gtk::Box) {
          set_orientation: gtk::Orientation::Vertical,

          // Content area.
          append: scroll = &gtk::ScrolledWindow {
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_width: true,
            set_propagate_natural_height: true,
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
          },
        },

        // Refresh button.
        append = reload = &gtk::Button {
          set_label: "Refresh",
          set_sensitive: watch! { !model.fetching },
          set_halign: gtk::Align::Center,
          set_css_classes: &["refresh"],
          connect_clicked[sender = components.async_handler.sender()] => move |_| {
            sender.blocking_send(AsyncHandlerMsg::Fetch).expect("Receiver dropped");
          }
        },

        // Status bar.
        append = status = &gtk::Statusbar {
          set_halign: gtk::Align::Fill,
        },
      },
    }
  }

  fn pre_view() {
    status.push(
      status.context_id("state"),
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

    let mut row_container = gtk::Box::builder().css_name("item").spacing(5);

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
      ForecastEntry::Future { day, night, .. } => {
        self.init_future_forecast(day.as_ref(), night.as_ref(), row_container);
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
  fn init_current_forecast(&self, forecast: &CurrentForecastWithEntry, row_container: &gtk::Box) {
    row_container.set_tooltip_markup(Some(&self.0.summary()));

    let info_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .spacing(5)
      .build();

    info_container.append(
      &gtk::Label::builder()
        .css_name("temperature")
        .css_classes(vec!["current".into()])
        .label(&if self.1 == TempUnit::Celsius {
          format!("{}", forecast.current.celsius)
        } else {
          format!("{}", forecast.current.fahrenheit)
        })
        .build(),
    );
    info_container.append(
      &gtk::Label::builder()
        .css_name("description")
        .label(&forecast.current.description)
        .build(),
    );
    row_container.append(&info_container);
  }

  /// Build widgets for a future forecast.
  fn init_future_forecast(
    &self,
    day: Option<&ForecastWithEntry>,
    night: Option<&ForecastWithEntry>,
    row_container: &gtk::Box,
  ) {
    row_container.set_tooltip_markup(Some(&self.0.summary()));
    if let Some(day) = day
      .or(night)
      .iter()
      .next()
      .map(|fc| fc.forecast.day_of_week.as_str())
    {
      row_container.append(
        &gtk::Label::builder()
          .label(day)
          .halign(gtk::Align::Center)
          .css_classes(vec!["dayofweek".into()])
          .build(),
      );
    }
    let day_night_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(5)
      .build();

    if day.is_none() {
      day_night_container.set_halign(gtk::Align::End);
    }

    if night.is_none() {
      day_night_container.set_halign(gtk::Align::Start);
    }

    for ForecastWithEntry { forecast, .. } in day.iter().chain(night.iter()) {
      day_night_container.append(&self.build_temp_and_title(forecast));
    }
    row_container.append(&day_night_container);
  }

  /// Build a temperature and title widgets for a day or night section.
  fn build_temp_and_title(&self, forecast: &Forecast) -> gtk::Box {
    let temp_label_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .spacing(5)
      .css_classes(vec!["tempAndDesc".into()])
      .build();

    let temp_string = match self.1 {
      TempUnit::Celsius => format!("{}", &forecast.celsius),
      TempUnit::Fahrenheit => format!("{}", &forecast.fahrenheit),
    };

    let high_low_label = gtk::Label::builder()
      .css_name("temperature")
      .css_classes(vec![match &forecast.celsius {
        Temperature::High(..) => "high".into(),
        Temperature::Low(..) => "low".into(),
        Temperature::Current(..) => "current".into(),
      }])
      .label(&temp_string)
      .justify(gtk::Justification::Right);

    let mut day_night_label = gtk::Label::builder()
      .css_name("description")
      .ellipsize(EllipsizeMode::End)
      .tooltip_text(&forecast.description)
      .label(&forecast.description);

    match forecast.day {
      DayNight::Day => {
        day_night_label = day_night_label
          .css_classes(vec!["day".into()])
          .halign(gtk::Align::Start)
      }
      DayNight::Night => day_night_label = day_night_label.css_classes(vec!["night".into()]),
    }

    temp_label_container.append(&high_low_label.build());
    temp_label_container.append(&day_night_label.build());

    temp_label_container
  }
}
