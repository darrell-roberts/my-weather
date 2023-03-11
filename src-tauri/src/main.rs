#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use my_weather::{
  get_weather,
  types::{to_forecast, ForecastEntry},
};
use serde::Serialize;
use std::time::Duration;
use tauri::{async_runtime::JoinHandle, Manager, Window};
use tokio::time;

#[derive(Serialize, Clone)]
struct LocalApiError(String);

impl std::fmt::Display for LocalApiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

#[tauri::command]
async fn get_weather_gui() -> Result<Vec<ForecastEntry>, LocalApiError> {
  get_weather()
    .await
    .map(|forecast| to_forecast(forecast.entries()))
    .map_err(|err| LocalApiError(err.to_string()))
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      start_refresh(app.get_window("main").expect("No main window"));
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![get_weather_gui])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

fn start_refresh(window: Window) -> JoinHandle<()> {
  tauri::async_runtime::spawn(async move {
    let mut interval = time::interval(Duration::from_secs(60 * 15));
    loop {
      interval.tick().await;
      get_weather()
        .await
        .map(|forecast| to_forecast(forecast.entries()))
        .map_err(|err| LocalApiError(err.to_string()))
        .and_then(|forecast| {
          window
            .emit("refresh", forecast)
            .map_err(|err| LocalApiError(err.to_string()))
        })
        .unwrap_or_else(|err| eprintln!("Failed to emit refresh {err}"))
    }
  })
}
