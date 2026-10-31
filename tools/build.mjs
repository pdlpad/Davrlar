import { build } from "esbuild";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const out = await build({
  entryPoints: [path.join(root, "client/main.js")],
  bundle: true, write: false, format: "iife", minify: true, target: "es2020",
  alias: { pixelfx: path.join(root, "vendor/pixelfx/src") },
  loader: { ".json": "json" },
});
let js = out.outputFiles[0].text.replace(/<\/script/gi, "<\\/script");
const html = readFileSync(path.join(root, "client/index.html"), "utf8").replace("/*SCRIPT*/", () => js);
mkdirSync(path.join(root, "dist"), { recursive: true });
writeFileSync(path.join(root, "dist/index.html"), html);
console.log("dist/index.html", (html.length / 1024).toFixed(0) + " KB");
