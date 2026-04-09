import { cpSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const frontendDir = path.resolve(scriptDir, "..");
const distDir = path.join(frontendDir, "dist");
const staticDir = path.resolve(frontendDir, "..", "static");

if (!existsSync(distDir)) {
  throw new Error(`未找到构建产物目录: ${distDir}`);
}

rmSync(staticDir, { recursive: true, force: true });
mkdirSync(staticDir, { recursive: true });
cpSync(distDir, staticDir, { recursive: true });
writeFileSync(path.join(staticDir, ".gitkeep"), "");

console.log("Copied dist to static/");
