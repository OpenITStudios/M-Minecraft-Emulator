import "./app.css";
import { mount } from "svelte";
import UnifiedManagerApp from "./UnifiedManagerApp.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("Minecraft Instance Manager UI root element is missing.");

mount(UnifiedManagerApp, { target });
