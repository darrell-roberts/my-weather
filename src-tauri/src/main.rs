#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use my_weather::{
  get_weather,
  types::{to_forecast, ForecastEntry},
};
use serde::Serialize;

#[derive(Serialize)]
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
    .invoke_handler(tauri::generate_handler![get_weather_gui])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
