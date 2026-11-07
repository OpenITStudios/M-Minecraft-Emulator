import { mount } from "svelte";
import LazyTesterApp from "./LazyTesterApp.svelte";

const target = document.getElementById("app");
if (!target) throw new Error("LazyTester UI root element is missing.");

mount(LazyTesterApp, { target });
