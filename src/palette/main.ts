import { mount } from "svelte";
import "../styles/app.css";
import App from "./App.svelte";
import { applyTheme, cachedTheme } from "../lib/theme";

// Paint the cached theme before mounting; Rust reconciles right after.
const cached = cachedTheme();
applyTheme(cached.theme, cached.accent);

const target = document.getElementById("app");

if (!target) {
  throw new Error("Palette root element #app is missing");
}

const app = mount(App, { target });

export default app;
