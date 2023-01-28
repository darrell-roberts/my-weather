import { CurrentEntry, ForecastEntry, FutureEntry, getDay, getNight } from "../../common/types";
import classes from "./Forecast.module.css";

type FutureForecastProps = {
  entry: ForecastEntry,
  unit: "Celsius" | "Fahrenheit"
}

export default function FutureForecast({ entry, unit = "Celsius" }: FutureForecastProps) {
  function renderTemperature(t: number): string {
    if (unit === "Celsius") {
      return `${t.toFixed(1)}°C`
    } else {
      return `${t.toFixed(0)}°F`
    }
  }

  const renderCurrent = (entry: CurrentEntry) => (
    <div className={classes.futureContainer}>
      <div className={classes.dayOfWeek}><span>Current</span></div>
      <div className={classes.current}>
        <div className={classes.temperature}>{renderTemperature(entry.current.celsius.content)}</div>
        <div className={classes.description}>{entry.current.description}</div>
      </div>
    </div>
  );

  function renderFuture(entry: FutureEntry) {
    const day = getDay(entry);
    const night = getNight(entry);
    return (
      <div className={classes.futureContainer}>
        <div className={classes.dayOfWeek}>
          <span>{entry.forecast[0].forecast["day_of_week"]}
          </span>
        </div>
        <div className={classes.future}>
          {day && <div className={classes.day}>
            <div className={classes.temperature}>{renderTemperature(day.forecast.celsius.content)}</div>
            <div className={classes.description}>{day.forecast.description}</div>
          </div>
          }
          {night && <div className={classes.night}>
            <div className={classes.temperature}>{renderTemperature(night.forecast.celsius.content)}</div>
            <div className={classes.description}>{night.forecast.description}</div>
          </div>
          }
        </div>
      </div>
    )
  }

  function renderEntry(entry: ForecastEntry) {
    switch (entry.type) {
      case "Current": return renderCurrent(entry.content);
      case "Future": return renderFuture(entry.content);
    }
  }

  return (
    <div className={classes.container}>
      {renderEntry(entry)}
    </div>
  )
}