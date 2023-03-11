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
      let window = app.get_window("main").expect("No main window");
      start_refresh(window);
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
      match get_weather()
        .await
        .map(|forecast| to_forecast(forecast.entries()))
        .map_err(|err| LocalApiError(err.to_string()))
      {
        Ok(forecast) => {
          window
            .emit("refresh", forecast)
            .unwrap_or_else(|err| eprintln!("Failed to emit refresh {err}"));
        }
        Err(err) => {
          window
            .emit("refresh_error", err)
            .unwrap_or_else(|err| eprintln!("Failed to emit refresh {err}"));
        }
      }
    }
  })
}
