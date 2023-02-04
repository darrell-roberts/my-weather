export type Forecast = {
  entry: Entry[];
}

export type Entry = {
  title: string;
  summary: string;
  category: Category;
}

export interface Category {
  term: "Weather Forecasts" | "Current Conditions";
}

export type ForecastEntry =
  { type: "Current"; content: CurrentEntry }
  | { type: "Future"; content: FutureEntry }
  | { type: "Warning"; content: WarningEntry };

export type CurrentEntry = {
  entry: { title: string, summary: string };
  current: {
    celsius: Temperature,
    fahrenheit: Temperature,
    description: string
  };
}

export type FutureEntry = {
  day: FutureDayNight;
  night: FutureDayNight;
}

export type FutureDayNight = {
  forecast: {
    celsius: Temperature;
    fahrenheit: Temperature;
    day: "Day" | "Night";
    day_of_week: DayOfWeek;
    description: string;
  },
  entry: Entry;
}

export type Temperature = {
  type: "High" | "Low" | "Current", content: number;
}

export type DayOfWeek = "Monday"
  | "Tuesday"
  | "Wednesday"
  | "Thursday"
  | "Friday"
  | "Saturday"
  | "Sunday";

export type WarningEntry = {
  title: string;
  summary: string;
}