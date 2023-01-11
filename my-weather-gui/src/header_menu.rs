use crate::{AppModel, AppMsg, TempUnit};
use relm4::{gtk::prelude::*, send, ComponentUpdate, Model, Widgets};

pub enum HeaderMsg {
  ChangeUnit(TempUnit),
}

pub struct HeaderModel {}

impl Model for HeaderModel {
  type Msg = HeaderMsg;
  type Widgets = HeaderWidgets;
  type Components = ();
}

impl ComponentUpdate<AppModel> for HeaderModel {
  fn init_model(_parent_model: &AppModel) -> Self {
    Self {}
  }

  fn update(
    &mut self,
    msg: Self::Msg,
    _components: &Self::Components,
    _sender: relm4::Sender<Self::Msg>,
    parent_sender: relm4::Sender<<AppModel as Model>::Msg>,
  ) {
    match msg {
      HeaderMsg::ChangeUnit(unit) => send!(parent_sender, AppMsg::ChangeUnit(unit)),
    }
  }
}

#[relm4::widget(pub)]
impl Widgets<HeaderModel, AppModel> for HeaderWidgets {
  view! {
    gtk::HeaderBar {
      set_title_widget = Some(&gtk::Box) {
        set_spacing: 5,
        set_halign: gtk::Align::Start,

        append = &gtk::Label {
            set_label: "My Weather",
        },

        append = &gtk::Box {
          append = group = &gtk::ToggleButton {
            set_label: "Celsius",
            set_active: true,
            connect_toggled(sender) => move |btn| {
              if btn.is_active() {
                send!(sender, HeaderMsg::ChangeUnit(TempUnit::Celsius));
              }
            },
          },

          append = &gtk::ToggleButton {
            set_label: "Fahrenheit",
            set_group: Some(&group),
            connect_toggled(sender) => move |btn| {
              if btn.is_active() {
                send!(sender, HeaderMsg::ChangeUnit(TempUnit::Fahrenheit));
              }
            }
          }
        },
      }
    }
  }
}