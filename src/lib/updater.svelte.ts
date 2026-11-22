// Checks GitHub for a newer MidiMaster, shows the update popup, and installs it.
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { getVersion } from "@tauri-apps/api/app";

const EVERY = 6 * 60 * 60 * 1000; // check again every 6 hours while running

let pending: Update | null = null; // plugin object; kept outside $state on purpose

export const upd = $state({
  current: "",
  latest: "",
  notes: "",
  show: false,
  checking: false,
  phase: "idle" as "idle" | "downloading" | "installing" | "error",
  done: 0,
  total: 0,
  message: "",
});

export async function checkForUpdate(manual = false) {
  if (upd.checking || upd.phase === "downloading" || upd.phase === "installing") return;
  upd.checking = true;
  upd.message = "";
  try {
    upd.current ||= await getVersion();
    const u = await check();
    if (u) {
      pending = u;
      upd.latest = u.version;
      upd.notes = u.body ?? "";
      upd.show = true;
      upd.phase = "idle";
    } else {
      upd.latest = "";
      if (manual) upd.message = `You have the latest version (${upd.current}).`;
    }
  } catch (e) {
    if (manual) upd.message = `Couldn't check for updates: ${e}`;
  } finally {
    upd.checking = false;
  }
}

export async function installUpdate() {
  if (!pending) return;
  upd.phase = "downloading";
  upd.done = 0;
  upd.total = 0;
  try {
    await pending.downloadAndInstall((ev) => {
      if (ev.event === "Started") upd.total = ev.data.contentLength ?? 0;
      else if (ev.event === "Progress") upd.done += ev.data.chunkLength;
      else if (ev.event === "Finished") upd.phase = "installing";
    });
    upd.phase = "installing";
    await relaunch(); // on Windows the installer closes and restarts the app itself
  } catch (e) {
    upd.phase = "error";
    upd.message = String(e);
  }
}

export function startUpdateChecks() {
  getVersion().then((v) => (upd.current = v)).catch(() => {});
  setTimeout(() => checkForUpdate(), 4000);
  setInterval(() => {
    if (!upd.show) checkForUpdate();
  }, EVERY);
}
