use my_weather_gui::{AppModel, Celsius};
use relm4::RelmApp;

fn main() {
  let app: RelmApp<AppModel<Celsius>> = RelmApp::new(AppModel::default());
  #[cfg(target_os = "macos")]
  relm4::set_global_css(include_bytes!("style_mac.css"));

  #[cfg(not(target_os = "macos"))]
  relm4::set_global_css(include_bytes!("style.css"));

  app.run();
}
