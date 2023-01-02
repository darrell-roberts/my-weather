use crate::{
  handlers::AsyncHandlerMsg, types::ForeCastEntry, AppModel, AppMsg,
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
      set_default_width: 300,
      set_default_height: 100,
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
            // set_spinning: watch! { model.fetching },
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
      println!("start spinner");
      spinner.start();
    } else {
      println!("stop spinner");
      spinner.stop();
    }
  }
}

/// Widgets used to for each [ForeCastEntry].
#[derive(Debug, Default)]
pub struct FactoryWidgets {
  _label: gtk::Label,
  _label_container: gtk::Box,
  _separator: gtk::Separator,
  container: gtk::Box,
}

impl FactoryPrototype for ForeCastEntry {
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
    let mut label_container = gtk::Box::builder()
      .css_name("item")
      .orientation(gtk::Orientation::Vertical)
      .halign(gtk::Align::Fill)
      .spacing(5);

    match self {
      Self::Current(_) => {
        label_container = label_container.css_classes(vec!["current".into()]);
      }
      Self::Warning(_) => {
        label_container = label_container.css_classes(vec!["warning".into()]);
      }
      _ => (),
    }

    let mut label = gtk::Label::builder()
      .halign(gtk::Align::Start)
      .justify(gtk::Justification::Left)
      .tooltip_markup(&self.summary());

    label = match self {
      Self::Warning(entry) => label.label(&entry.title),
      Self::Current(entry) => label.label(&entry.title),
      Self::Future { day, night, .. } => match (day, night) {
        (Some(d), Some(n)) => label.label(&format!("{d}\n{n}")),
        (Some(d), None) => label.label(&d.title),
        (None, Some(n)) => label.label(&n.title),
        _ => label,
      },
    };

    let label = label.build();

    let separator = gtk::Separator::builder()
      .orientation(gtk::Orientation::Horizontal)
      .halign(gtk::Align::Fill)
      .hexpand(true)
      .build();

    let container = gtk::Box::builder()
      .orientation(gtk::Orientation::Vertical)
      .spacing(5)
      .build();

    let label_container = label_container.build();

    label_container.append(&label);

    container.append(&label_container);
    container.append(&separator);

    FactoryWidgets {
      container,
      _label: label,
      _label_container: label_container,
      _separator: separator,
    }
  }

  fn position(
    &self,
    _key: &<Self::Factory as relm4::factory::Factory<Self, Self::View>>::Key,
  ) -> <Self::View as relm4::factory::FactoryView<Self::Root>>::Position {
  }

  fn view(
    &self,
    _key: &<Self::Factory as relm4::factory::Factory<Self, Self::View>>::Key,
    _widgets: &Self::Widgets,
  ) {
  }

  fn root_widget(widgets: &Self::Widgets) -> &Self::Root {
    &widgets.container
  }
}
