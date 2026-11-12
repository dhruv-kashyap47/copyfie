import { mount } from "svelte";
import "../styles/app.css";
import App from "./App.svelte";

const target = document.getElementById("app");

if (!target) {
  throw new Error("Palette root element #app is missing");
}

const app = mount(App, { target });

export default app;
