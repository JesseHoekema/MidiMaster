import { mount } from "svelte";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";
import App from "./App.svelte";
import "./app.css";

mount(App, { target: document.getElementById("app")! });
