import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import CustomCursor from "./components/common/CustomCursor";
import { useI18nStore } from "./i18n";
import { useSettingsStore } from "./stores/useSettingsStore";
import "./styles/globals.css";

void useI18nStore.getState().initLocale();
void useSettingsStore.getState().initSettings();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <CustomCursor />
    <App />
  </React.StrictMode>,
);
