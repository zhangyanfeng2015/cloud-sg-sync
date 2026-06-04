import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const version = JSON.parse(
  readFileSync(join(root, "package.json"), "utf8"),
).version;
const tag = `v${version}`;

const exists = spawnSync("git", ["rev-parse", tag], {
  cwd: root,
  stdio: "ignore",
});
if (exists.status === 0) {
  console.log(`[release:tag] 标签 ${tag} 已存在，跳过`);
  process.exit(0);
}

const r = spawnSync("git", ["tag", "-a", tag, "-m", `release: ${tag}`], {
  cwd: root,
  stdio: "inherit",
});
if (r.status !== 0) process.exit(r.status ?? 1);

console.log(`[release:tag] 已创建 ${tag}`);
console.log(`  git push origin ${tag}`);
console.log(`  git push gitee ${tag}`);
