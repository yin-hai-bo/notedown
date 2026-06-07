import { createApp } from "vue";
import App from "./App.vue";
import { initializeSettings, watchSettingsChanges } from "./settings";

async function bootstrap(): Promise<void> {
  await initializeSettings();
  await watchSettingsChanges();
  createApp(App).mount("#app");
}

bootstrap().catch((error) => {
  console.error("Failed to initialize application settings", error);
  createApp(App).mount("#app");
});
