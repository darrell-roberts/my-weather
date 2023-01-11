//! An error dialog component.
use crate::{widgets::AppWidgets, AppModel};
use relm4::{gtk::prelude::*, send, ComponentUpdate, Model, Widgets};

pub struct ErrorDialogModel {
  hidden: bool,
  message: Option<String>,
}

pub enum DialogMsg {
  Open(String),
  Close,
}

impl ComponentUpdate<AppModel> for ErrorDialogModel {
  fn init_model(_parent_model: &AppModel) -> Self {
    Self {
      hidden: true,
      message: None,
    }
  }
  fn update(
    &mut self,
    msg: Self::Msg,
    _components: &Self::Components,
    _sender: relm4::Sender<Self::Msg>,
    _parent_sender: relm4::Sender<<AppModel as relm4::Model>::Msg>,
  ) {
    match msg {
      DialogMsg::Open(text) => {
        self.message = Some(text);
        self.hidden = false;
      }
      DialogMsg::Close => self.hidden = true,
    }
  }
}

pub struct ErrorDialogWidgets {
  dialog: gtk::MessageDialog,
}

impl Widgets<ErrorDialogModel, AppModel> for ErrorDialogWidgets {
  type Root = gtk::MessageDialog;

  fn init_view(
    _model: &ErrorDialogModel,
    _components: &<ErrorDialogModel as Model>::Components,
    sender: relm4::Sender<<ErrorDialogModel as Model>::Msg>,
  ) -> Self {
    let dialog = gtk::MessageDialog::builder()
      .modal(true)
      .visible(false)
      .message_type(gtk::MessageType::Error)
      .build();

    dialog.add_button("Close", gtk::ResponseType::Close);
    dialog.connect_response(move |_, _| send!(sender, DialogMsg::Close));

    Self { dialog }
  }

  fn connect_parent(&mut self, parent_widgets: &<AppModel as Model>::Widgets) {
    self
      .dialog
      .set_transient_for(Some(&<AppWidgets as Widgets<AppModel, ()>>::root_widget(
        parent_widgets,
      )))
  }

  fn root_widget(&self) -> Self::Root {
    self.dialog.clone()
  }

  fn view(
    &mut self,
    model: &ErrorDialogModel,
    _sender: relm4::Sender<<ErrorDialogModel as Model>::Msg>,
  ) {
    self.dialog.set_visible(!model.hidden);
    self.dialog.set_text(model.message.as_deref());
  }
}

impl Model for ErrorDialogModel {
  type Components = ();
  type Widgets = ErrorDialogWidgets;
  type Msg = DialogMsg;
}
