import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  // Do not copy an arbitrary public/ tree into the embedded frontend.
  publicDir: false,
  server: {
    host: "127.0.0.1", port: 1421, strictPort: true,
    // Rust build output can contain locked PDB files on Windows.
    watch: { ignored: ["**/src-tauri/target/**"] }
  },
  build: { outDir: "dist", emptyOutDir: true }
});
