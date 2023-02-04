import { CurrentEntry, ForecastEntry, FutureEntry, WarningEntry, } from "../../common/types";
import Tooltip from "../Tooltip/Tooltip";
import classes from "./Forecast.module.css";

type FutureForecastProps = {
  entry: ForecastEntry,
  unit: "Celsius" | "Fahrenheit"
}

export default function FutureForecast({ entry, unit = "Celsius" }: FutureForecastProps) {
  function renderTemperature(t: number, fractionDigits?: boolean): string {
    if (unit === "Celsius") {
      return `${fractionDigits ? t.toFixed(1) : t}°`
    } else {
      return `${t}°F`
    }
  }

  const renderCurrent = (entry: CurrentEntry) => (
    <Tooltip message={entry.entry.summary}>
      <div className={classes.futureContainer}>
        <div className={classes.dayOfWeek}><span>Current</span></div>
        <div className={classes.current}>
          <div className={classes.temperature}>{renderTemperature(entry.current.celsius.content, true)}</div>
          <div className={classes.description}>{entry.current.description}</div>
        </div>
      </div>
    </Tooltip>
  );

  function renderFuture(entry: FutureEntry) {
    const dayOfweek = entry.day?.forecast.day_of_week ?? entry.night?.forecast.day_of_week;
    return (
      <div className={classes.futureContainer}>
        <div className={classes.dayOfWeek}>
          <span>{dayOfweek}
          </span>
        </div>
        <div className={classes.future}>
          {entry.day &&
            <Tooltip message={entry.day.entry.summary}>
              <div className={classes.day}>
                <div className={classes.temperature}>{renderTemperature(entry.day.forecast.celsius.content)}</div>
                <div className={classes.description}>{entry.day.forecast.description}</div>
              </div>
            </Tooltip>

          }
          {entry.night &&
            <Tooltip message={entry.night.entry.summary}>
              <div className={classes.night}>
                <div className={classes.temperature}>{renderTemperature(entry.night.forecast.celsius.content)}</div>
                <div className={classes.description}>{entry.night.forecast.description}</div>
              </div>
            </Tooltip>
          }
        </div>
      </div>
    )
  }

  function renderWarning(entry: WarningEntry) {
    return (
      <div className={classes.warningContainer}>
        <Tooltip message={entry.summary}>
          <div>{entry.title}</div>
        </Tooltip>
      </div>
    )
  }

  function renderEntry(entry: ForecastEntry) {
    switch (entry.type) {
      case "Current": return renderCurrent(entry.content);
      case "Future": return renderFuture(entry.content);
      case "Warning": return renderWarning(entry.content);
    }
  }

  return (
    <div className={classes.container}>
      {renderEntry(entry)}
    </div>
  )
}