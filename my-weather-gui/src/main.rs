use my_weather_gui::AppModel;
use relm4::RelmApp;

fn main() {
  let app = RelmApp::new(AppModel::default());
  relm4::set_global_css(include_bytes!("style.css"));
  app.run();
}
