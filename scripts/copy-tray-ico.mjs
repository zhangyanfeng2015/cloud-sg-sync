import { copyFileSync, rmSync } from "node:fs";

copyFileSync("src-tauri/icons/tray-out/icon.ico", "src-tauri/icons/tray.ico");
rmSync("src-tauri/icons/tray-out", { recursive: true, force: true });
