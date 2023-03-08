mod components;
pub use components::AppModel;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TempUnit {
  Fahrenheit,
  #[default]
  Celsius,
}
