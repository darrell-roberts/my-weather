import { useEffect, useReducer } from 'react'
import './App.css'
import './components/FutureForecast/Forecast';
import FutureForecast from './components/FutureForecast/Forecast';
import { invoke } from '@tauri-apps/api';
import { ForecastEntry } from "./common/types";

type AppState = {
  entries: ForecastEntry[];
  fetching: boolean;
  error?: string;
}

type Action = { type: "getWeather" }
  | { type: "error", error: string }
  | { type: "receiveWeather", weather: ForecastEntry[] };

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case "getWeather": return {
      ...state,
      fetching: true,
      error: undefined,
    }
    case "error": return {
      ...state,
      fetching: false,
      error: action.error
    }
    case "receiveWeather": return {
      ...state,
      fetching: false,
      entries: action.weather
    }
  }
}

const INITIAL_STATE: AppState = { fetching: false, entries: [] };

function App() {
  const [state, dispatch] = useReducer(reducer, INITIAL_STATE);

  useEffect(() => {
    dispatch({ type: "getWeather" });
  }, []);

  useEffect(() => {
    if (state.fetching) {
      invoke<ForecastEntry[]>("get_weather_gui")
        .then(data => dispatch({ type: "receiveWeather", weather: data }))
        .catch(error => dispatch({ type: "error", error: error.toString() }));
    }
  }, [state.fetching])

  function renderForeCast() {
    if (!state.fetching && state.entries.length > 0) {
      return state.entries.map(entry => <FutureForecast entry={entry} unit="Celsius" />)
    } else {
      return <div><span className="loading">Loading...</span></div>
    }
  }

  return (
    <div className="App">
      {renderForeCast()}
      <button
        className="refresh"
        disabled={state.fetching}
        onClick={() => dispatch({ type: "getWeather" })}>
        Refresh
      </button>
    </div>
  )
}

export default App
