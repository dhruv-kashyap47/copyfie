import { mount } from "svelte";
import "../styles/settings.css";
import App from "./App.svelte";
import { applyTheme, cachedTheme } from "../lib/theme";

const cached = cachedTheme();
applyTheme(cached.theme, cached.accent);

const target = document.getElementById("settings");

if (!target) {
  throw new Error("Settings root element #settings is missing");
}

const app = mount(App, { target });

export default app;
