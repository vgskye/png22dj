import { defineConfig } from "astro/config";
import wasm from "vite-plugin-wasm";
import tailwind from "@astrojs/tailwind";

import icon from "astro-icon";

// https://astro.build/config
export default defineConfig({
  vite: {
    plugins: [wasm()],
  },
  integrations: [tailwind(), icon()],
});
