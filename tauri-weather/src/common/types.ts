export interface Forecast {
  entry: Entry[]
}

export interface Entry {
  title: string;
  summary: string;
  category: Category;
}

export interface Category {
  term: "Weather Forecasts" | "Current Conditions"
}

export type ForecastEntry =
  { type: "Current"; content: CurrentEntry }
  | { type: "Future"; content: FutureEntry };

export type CurrentEntry = {
  entry: { title: string, summary: string };
  current: { celsius: Temperature, fahrenheit: Temperature, description: string }
}

export type FutureEntry = {
  forecast: FutureDayNight[]
}

export type FutureDayNight = {
  forecast: {
    celsius: Temperature,
    fahrenheit: Temperature,
    day: "Day" | "Night",
    day_of_week: DayOfWeek,
    description: string,
  }
}

export type Temperature = {
  type: "High" | "Low" | "Current", content: number
}

export type DayOfWeek = "Monday"
  | "Tuesday"
  | "Wednesday"
  | "Thursday"
  | "Friday"
  | "Saturday"
  | "Sunday";


export const getDay = (entry: FutureEntry) => entry.forecast.find(el => el.forecast.day === "Day");
export const getNight = (entry: FutureEntry) => entry.forecast.find(el => el.forecast.day === "Night");
