use super::app::AppMsg;
use crate::TempUnit;
use gtk::pango::EllipsizeMode;
use my_weather::types::{
  CurrentForecastWithEntry, DayNight, Forecast, ForecastEntry, ForecastWithEntry, Temperature,
};
use relm4::{gtk::prelude::*, prelude::FactoryComponent};

#[derive(Debug)]
pub struct ForecastEntryAndTempUnit(pub ForecastEntry, pub TempUnit);

pub enum ForecastWidgets {
  Future {
    day_temp_label: Option<gtk::Label>,
    night_temp_label: Option<gtk::Label>,
  },
  Current {
    temperature_label: gtk::Label,
  },
  None,
}

impl FactoryComponent for ForecastEntryAndTempUnit {
  type Init = (ForecastEntry, TempUnit);
  type Input = ();
  type Output = ();
  type ParentWidget = gtk::Box;
  type ParentInput = AppMsg;
  type CommandOutput = ();
  type Widgets = ForecastWidgets;
  type Root = gtk::Box;

  fn init_root(&self) -> Self::Root {
    gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(5)
      .build()
  }

  fn init_widgets(
    &mut self,
    _index: &relm4::prelude::DynamicIndex,
    root: &Self::Root,
    _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
    _sender: relm4::FactorySender<Self>,
  ) -> Self::Widgets {
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
    let widgets = self.init_forecast(&row_container);
    root.append(&row_container);
    let separator = gtk::Separator::builder()
      .orientation(gtk::Orientation::Horizontal)
      .build();
    root.append(&separator);
    widgets
  }

  fn init_model(
    (entry, unit): Self::Init,
    _index: &relm4::prelude::DynamicIndex,
    _sender: relm4::FactorySender<Self>,
  ) -> Self {
    Self(entry, unit)
  }

  fn output_to_parent_input(_output: Self::Output) -> Option<Self::ParentInput> {
    None
  }

  fn update_view(&self, widgets: &mut Self::Widgets, _sender: relm4::FactorySender<Self>) {
    match widgets {
      ForecastWidgets::Future {
        day_temp_label: day_label,
        night_temp_label: night_label,
      } => {
        if let ForecastEntry::Future { day, night, .. } = &self.0 {
          let update_units =
            |(dn, dn_label): (Option<&ForecastWithEntry>, Option<&mut gtk::Label>)| {
              if let (Some(dn), Some(dn_label)) = (dn, dn_label) {
                let temp_string = match self.1 {
                  TempUnit::Celsius => format!("{}", &dn.forecast.celsius),
                  TempUnit::Fahrenheit => format!("{}", &dn.forecast.fahrenheit),
                };
                dn_label.set_label(&temp_string);
              }
            };
          update_units((day.as_ref(), day_label.as_mut()));
          update_units((night.as_ref(), night_label.as_mut()));
        }
      }
      ForecastWidgets::Current { temperature_label } => {
        if let ForecastEntry::Current(forecast) = &self.0 {
          temperature_label.set_label(&if self.1 == TempUnit::Celsius {
            format!("{}", forecast.current.celsius)
          } else {
            format!("{}", forecast.current.fahrenheit)
          })
        }
      }
      ForecastWidgets::None => (),
    }
  }
}

impl ForecastEntryAndTempUnit {
  /// Build widgets inside a forecast container.
  fn init_forecast(&self, row_container: &gtk::Box) -> ForecastWidgets {
    match &self.0 {
      ForecastEntry::Future { day, night, .. } => {
        self.init_future_forecast(day.as_ref(), night.as_ref(), row_container)
      }
      ForecastEntry::Current(forecast) => self.init_current_forecast(forecast, row_container),
      ForecastEntry::Warning(entry) => {
        row_container.append(
          &gtk::Label::builder()
            .halign(gtk::Align::Center)
            .tooltip_markup(&self.0.summary())
            .label(&entry.title)
            .build(),
        );
        ForecastWidgets::None
      }
    }
  }

  /// Build widgets for a current forecast.
  fn init_current_forecast(
    &self,
    forecast: &CurrentForecastWithEntry,
    row_container: &gtk::Box,
  ) -> ForecastWidgets {
    row_container.set_tooltip_markup(Some(&self.0.summary()));

    let info_container = gtk::Box::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Center)
      .spacing(5)
      .build();

    let temperature_label = gtk::Label::builder()
      .css_name("temperature")
      .css_classes(vec!["current".into()])
      .label(&if self.1 == TempUnit::Celsius {
        format!("{}", forecast.current.celsius)
      } else {
        format!("{}", forecast.current.fahrenheit)
      })
      .build();

    info_container.append(&temperature_label);
    info_container.append(
      &gtk::Label::builder()
        .css_name("description")
        .label(&forecast.current.description)
        .build(),
    );
    row_container.append(&info_container);

    ForecastWidgets::Current { temperature_label }
  }

  /// Build widgets for a future forecast.
  fn init_future_forecast(
    &self,
    day: Option<&ForecastWithEntry>,
    night: Option<&ForecastWithEntry>,
    row_container: &gtk::Box,
  ) -> ForecastWidgets {
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

    let day_night_labels = |dn: Option<&ForecastWithEntry>| -> Option<gtk::Label> {
      dn.map(|ForecastWithEntry { forecast, .. }| {
        let (temp_and_title, temp_label) = self.build_temp_and_title(forecast);
        day_night_container.append(&temp_and_title);
        temp_label
      })
    };

    let day_temp_label = day_night_labels(day);
    let night_temp_label = day_night_labels(night);

    row_container.append(&day_night_container);

    ForecastWidgets::Future {
      day_temp_label,
      night_temp_label,
    }
  }

  /// Build a temperature and title widgets for a day or night section.
  fn build_temp_and_title(&self, forecast: &Forecast) -> (gtk::Box, gtk::Label) {
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
      .justify(gtk::Justification::Right)
      .build();

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

    temp_label_container.append(&high_low_label);
    temp_label_container.append(&day_night_label.build());

    (temp_label_container, high_low_label)
  }
}
