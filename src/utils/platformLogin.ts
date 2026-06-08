import { invoke } from "@tauri-apps/api/core";

export type PlatformLoginOpenResult = "opened" | "has_session" | "failed";

export async function hasPlatformSession(): Promise<boolean> {
  try {
    return await invoke<boolean>("has_platform_session");
  } catch {
    return false;
  }
}

export async function openPlatformLogin(): Promise<void> {
  await invoke("open_platform_login");
}

export async function openPlatformLoginIfNeeded(): Promise<PlatformLoginOpenResult> {
  try {
    const hasSession = await hasPlatformSession();
    if (!hasSession) {
      await openPlatformLogin();
      return "opened";
    }
    return "has_session";
  } catch (err) {
    console.error("open_platform_login failed:", err);
    return "failed";
  }
}
