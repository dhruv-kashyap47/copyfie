import { mount } from "svelte";
import "../styles/app.css";
import App from "./App.svelte";
import { readBootstrap } from "../lib/bootstrap";
import { applyTheme, cachedTheme } from "../lib/theme";

// Paint the theme before mounting. A cold bootstrap carries the authoritative
// values; otherwise use the localStorage cache and reconcile after mount.
const boot = readBootstrap();
if (boot) {
  applyTheme(boot.theme, boot.accent);
} else {
  const cached = cachedTheme();
  applyTheme(cached.theme, cached.accent);
}

const target = document.getElementById("app");

if (!target) {
  throw new Error("Palette root element #app is missing");
}

const app = mount(App, { target });

export default app;
