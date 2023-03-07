//! An error dialog component.
use relm4::{gtk::prelude::*, ComponentParts, SimpleComponent};

pub struct ErrorDialogModel {
  hidden: bool,
  message: Option<String>,
}

#[derive(Debug)]
pub enum DialogMsg {
  Open(String),
  Close,
}

#[relm4::component(pub)]
impl SimpleComponent for ErrorDialogModel {
  type Widgets = DialogWidgets;
  type Init = ();
  type Input = DialogMsg;
  type Output = ();

  view! {
    gtk::MessageDialog {
      set_modal: true,
      #[watch]
      set_visible: !model.hidden,
      #[watch]
      set_text: model.message.as_deref(),
      set_message_type: gtk::MessageType::Error,
      add_button: ("Close", gtk::ResponseType::Close),
      connect_response[sender] => move |_, _| {
        sender.input(DialogMsg::Close)
      }

    }
  }

  fn init(
    _init: Self::Init,
    root: &Self::Root,
    sender: relm4::ComponentSender<Self>,
  ) -> relm4::ComponentParts<Self> {
    let model = ErrorDialogModel {
      hidden: true,
      message: None,
    };

    let widgets = view_output!();
    ComponentParts { model, widgets }
  }

  fn update(&mut self, message: Self::Input, _sender: relm4::ComponentSender<Self>) {
    match message {
      DialogMsg::Open(text) => {
        self.message = Some(text);
        self.hidden = false;
      }
      DialogMsg::Close => self.hidden = true,
    }
  }
}

// pub struct ErrorDialogWidgets {
//   dialog: gtk::MessageDialog,
// }

// impl Widgets<ErrorDialogModel, AppModel> for ErrorDialogWidgets {
//   type Root = gtk::MessageDialog;

//   fn init_view(
//     _model: &ErrorDialogModel,
//     _components: &<ErrorDialogModel as Model>::Components,
//     sender: relm4::Sender<<ErrorDialogModel as Model>::Msg>,
//   ) -> Self {
//     let dialog = gtk::MessageDialog::builder()
//       .modal(true)
//       .visible(false)
//       .message_type(gtk::MessageType::Error)
//       .build();

//     dialog.add_button("Close", gtk::ResponseType::Close);
//     dialog.connect_response(move |_, _| send!(sender, DialogMsg::Close));

//     Self { dialog }
//   }

//   fn connect_parent(&mut self, parent_widgets: &<AppModel as Model>::Widgets) {
//     self
//       .dialog
//       .set_transient_for(Some(&<AppWidgets as Widgets<AppModel, ()>>::root_widget(
//         parent_widgets,
//       )))
//   }

//   fn root_widget(&self) -> Self::Root {
//     self.dialog.clone()
//   }

//   fn view(
//     &mut self,
//     model: &ErrorDialogModel,
//     _sender: relm4::Sender<<ErrorDialogModel as Model>::Msg>,
//   ) {
//     self.dialog.set_visible(!model.hidden);
//     self.dialog.set_text(model.message.as_deref());
//   }
// }

// impl Model for ErrorDialogModel {
//   type Components = ();
//   type Widgets = ErrorDialogWidgets;
//   type Msg = DialogMsg;
// }
