import { readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type BudgetRule = {
  prefix: string;
  limitKb: number;
};

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const frontendDir = path.resolve(scriptDir, "..");
const assetsDir = path.join(frontendDir, "dist", "assets");

const namedBudgets: BudgetRule[] = [
  { prefix: "lib-react-dom-", limitKb: 192 },
  { prefix: "lib-framer-motion-", limitKb: 144 },
  { prefix: "lib-react-router-", limitKb: 96 },
];
const defaultLimitKb = 64;

function toLimitBytes(limitKb: number): number {
  return limitKb * 1024;
}

function toDisplayKb(sizeBytes: number): string {
  return (sizeBytes / 1024).toFixed(2);
}

const jsFiles = readdirSync(assetsDir)
  .filter((entry) => entry.endsWith(".js"))
  .sort((left, right) => left.localeCompare(right));

const failures: string[] = [];

for (const fileName of jsFiles) {
  const filePath = path.join(assetsDir, fileName);
  const sizeBytes = statSync(filePath).size;
  const matchedBudget = namedBudgets.find((rule) => fileName.startsWith(rule.prefix));
  const limitKb = matchedBudget?.limitKb ?? defaultLimitKb;
  const limitBytes = toLimitBytes(limitKb);

  if (sizeBytes > limitBytes) {
    failures.push(
      `${fileName} 实际 ${toDisplayKb(sizeBytes)}KB，超过 ${limitKb}KB 上限`
    );
  }
}

if (failures.length > 0) {
  throw new Error(
    `前端 bundle 预算检查失败:\n${failures.map((item) => `- ${item}`).join("\n")}`
  );
}

console.log(`bundle 预算检查通过，共检查 ${jsFiles.length} 个 JS chunk`);
