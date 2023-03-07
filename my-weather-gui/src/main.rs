use my_weather::get_weather;
use my_weather_gui::AppModel;
use relm4::RelmApp;

#[tokio::main]
async fn main() {
  let app = RelmApp::new("dr.weather");
  #[cfg(target_os = "macos")]
  relm4::set_global_css(include_str!("style_mac.css"));

  #[cfg(not(target_os = "macos"))]
  relm4::set_global_css(include_str!("style.css"));

  app.run::<AppModel>(get_weather().await);
}
