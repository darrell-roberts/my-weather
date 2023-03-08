//! A header component with title and menu button.
use crate::TempUnit;
use relm4::{gtk::prelude::*, ComponentParts, SimpleComponent};

#[derive(Debug)]
pub enum HeaderMsg {
  ChangeUnit(TempUnit),
}

pub struct HeaderModel;

#[relm4::component(pub)]
impl SimpleComponent for HeaderModel {
  type Init = ();
  type Input = ();
  type Output = HeaderMsg;

  view! {
    #[root]
    gtk::HeaderBar {
      #[wrap(Some)]
      set_title_widget = &gtk::Box {
        set_spacing: 5,
        set_halign: gtk::Align::End,

        gtk::Label {
          set_markup: "<b>My Weather</b>",
        },

        #[name = "button"]
        gtk::MenuButton {
          #[wrap(Some)]
          set_popover = &gtk::Popover {
            #[wrap(Some)]
            set_child = &gtk::Box {
              set_orientation: gtk::Orientation::Vertical,
              set_spacing: 10,
              #[name = "group"]
              gtk::ToggleButton {
                set_label: "Celsius",
                set_active: true,
                connect_toggled[sender, button] => move |btn| {
                    if btn.is_active() {
                        sender.output(HeaderMsg::ChangeUnit(TempUnit::Celsius)).unwrap();
                        button.popdown();
                    }
                }
              },
              gtk::ToggleButton {
                set_label: "Fahrenheit",
                set_group: Some(&group),
                connect_toggled[sender, button] => move |btn| {
                    if btn.is_active() {
                        sender.output(HeaderMsg::ChangeUnit(TempUnit::Fahrenheit)).unwrap();
                        button.popdown();
                    }
                }
              }
            }
          },
        },
      }
    }
  }

  fn init(
    _init: Self::Init,
    root: &Self::Root,
    sender: relm4::ComponentSender<Self>,
  ) -> relm4::ComponentParts<Self> {
    let model = HeaderModel;
    let widgets = view_output!();

    ComponentParts { model, widgets }
  }
}
