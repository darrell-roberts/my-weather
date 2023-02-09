import { CurrentEntry, ForecastEntry, FutureEntry, WarningEntry, } from "../../common/types";
import Tooltip from "../Tooltip/Tooltip";
import classes from "./Forecast.module.css";

type FutureForecastProps = {
    entry: ForecastEntry;
    unit: Unit;
}

type Unit = "Celsius" | "Fahrenheit";

export const FutureForecast =
    ({ entry, unit = "Celsius" }: FutureForecastProps) =>
        <div className={classes.container}>
            {renderEntry(entry, unit)}
        </div>

const renderEntry = (entry: ForecastEntry, unit: Unit) => {
    switch (entry.type) {
        case "Current": return renderCurrent(entry.content, unit);
        case "Future": return renderFuture(entry.content, unit);
        case "Warning": return renderWarning(entry.content);
    }
}

const renderWarning = (entry: WarningEntry) =>
    <div className={classes.warningContainer}>
        <Tooltip message={entry.summary}>
            <div>{entry.title}</div>
        </Tooltip>
    </div>

function renderFuture(entry: FutureEntry, unit: Unit) {
    const dayOfweek = entry.day?.forecast.day_of_week
        ?? entry.night?.forecast.day_of_week;

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
                            <div className={classes.temperature}>
                                {renderTemperature(entry.day.forecast.celsius.content, unit)}
                            </div>
                            <div className={classes.description}>
                                {entry.day.forecast.description}
                            </div>
                        </div>
                    </Tooltip>
                }
                {entry.night &&
                    <Tooltip message={entry.night.entry.summary}>
                        <div className={classes.night}>
                            <div className={classes.temperature}>
                                {renderTemperature(entry.night.forecast.celsius.content, unit)}
                            </div>
                            <div className={classes.description}>
                                {entry.night.forecast.description}
                            </div>
                        </div>
                    </Tooltip>
                }
            </div>
        </div>
    )
}

function renderTemperature(t: number, unit: Unit, fractionDigits?: boolean): string {
    if (unit === "Celsius") {
        return `${fractionDigits ? t.toFixed(1) : t}°`
    } else {
        return `${t}°F`
    }
}

const renderCurrent = (entry: CurrentEntry, unit: Unit) => (
    <Tooltip htmlMessage={entry.entry.summary}>
        <div className={classes.futureContainer}>
            <div className={classes.dayOfWeek}>
                <span>Now</span>
            </div>
            <div className={classes.current}>
                <div className={classes.temperature}>
                    {renderTemperature(entry.current.celsius.content, unit, true)}
                </div>
                <div className={classes.description}>
                    {entry.current.description}
                </div>
            </div>
        </div>
    </Tooltip>
);

export default FutureForecast;