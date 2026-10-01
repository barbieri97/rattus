import { createApp, type Component } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./style.css";

async function root(): Promise<Component> {
  // Development aid: the dev server with ?gallery shows every pose of the rat.
  if (import.meta.env.DEV && new URLSearchParams(location.search).has("gallery")) {
    return (await import("./dev/PoseGallery.vue")).default;
  }
  return App;
}

void root().then((component) => createApp(component).use(createPinia()).mount("#app"));
