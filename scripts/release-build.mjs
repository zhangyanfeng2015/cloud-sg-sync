import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const version = pkg.version;
const tag = `v${version}`;

function run(cmd, args) {
  const r = spawnSync(cmd, args, { cwd: root, stdio: "inherit", shell: true });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

console.log(`[release] 构建 Cloud SG Sync ${tag} …`);
run("pnpm", ["tauri", "build"]);

const nsisDir = join(root, "src-tauri", "target", "release", "bundle", "nsis");
if (!existsSync(nsisDir)) {
  console.error(`[release] 未找到目录: ${nsisDir}`);
  process.exit(1);
}

const installers = readdirSync(nsisDir).filter((f) => /\.exe$/i.test(f));
if (installers.length === 0) {
  console.error(`[release] nsis 目录下无 .exe: ${nsisDir}`);
  process.exit(1);
}

const srcExe = join(nsisDir, installers[0]);
const outDir = join(root, "release", tag);
mkdirSync(outDir, { recursive: true });

const outExe = join(outDir, installers[0]);
copyFileSync(srcExe, outExe);

const notesSrc = join(root, "release-notes", `${version}.md`);
const notesOut = join(outDir, "RELEASE_NOTES.md");
if (existsSync(notesSrc)) {
  copyFileSync(notesSrc, notesOut);
} else {
  writeFileSync(
    notesOut,
    `# Cloud SG Sync ${tag}\n\n请补充 release-notes/${version}.md 后重新执行 pnpm run release。\n`,
    "utf8",
  );
}

const manifest = {
  version,
  tag,
  builtAt: new Date().toISOString(),
  installer: outExe,
  releaseNotes: notesOut,
  gitTagCommand: `git tag -a ${tag} -m "release: ${tag}"`,
  gitPushTags: `git push origin ${tag} && git push gitee ${tag}`,
};

writeFileSync(join(outDir, "manifest.json"), JSON.stringify(manifest, null, 2), "utf8");

console.log("");
console.log("[release] 完成");
console.log(`  安装包: ${outExe}`);
console.log(`  说明:   ${notesOut}`);
console.log(`  清单:   ${join(outDir, "manifest.json")}`);
console.log("");
console.log("上传到 GitHub / Gitee Release 时：");
console.log(`  1. 标签使用 ${tag}（若未打标签: ${manifest.gitTagCommand}）`);
console.log(`  2. 上传安装包: ${installers[0]}`);
console.log(`  3. 发布说明可复制: ${notesOut}`);
console.log(`  4. 推送标签: ${manifest.gitPushTags}`);
