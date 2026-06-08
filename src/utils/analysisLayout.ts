import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

export function clearLegacyCanvasPref(): void {
  try {
    localStorage.removeItem("analysis_canvas_expanded");
    localStorage.removeItem("analysis_side_preference");
  } catch {
    // ignore
  }
}

export async function setLayoutTransitioning(active: boolean): Promise<void> {
  try {
    await invoke("set_layout_transitioning", { active });
  } catch {
    // ignore in dev/browser
  }
}

export async function setWindowLogicalSize(
  width: number,
  height: number,
): Promise<void> {
  try {
    const win = getCurrentWindow();
    await win.setSize(new LogicalSize(width, height));
  } catch {
    // ignore in dev/browser
  }
}

export function waitPanelTransition(
  el: HTMLElement | null,
  ms: number,
): Promise<void> {
  if (!el) {
    return new Promise((resolve) => window.setTimeout(resolve, ms));
  }

  return new Promise((resolve) => {
    let settled = false;
    const finish = () => {
      if (settled) return;
      settled = true;
      resolve();
    };

    const timer = window.setTimeout(finish, ms + 80);

    el.addEventListener(
      "transitionend",
      (event) => {
        if (event.target !== el) return;
        window.clearTimeout(timer);
        finish();
      },
      { once: true },
    );
  });
}

export async function animateDrawerOpen(
  setMounted: (value: boolean) => void,
  setVisible: (value: boolean) => void,
  getEl: () => HTMLElement | null,
  ms: number,
): Promise<void> {
  setMounted(true);
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  });
  setVisible(true);
  await waitPanelTransition(getEl(), ms);
}

export async function animateDrawerClose(
  setVisible: (value: boolean) => void,
  setMounted: (value: boolean) => void,
  getEl: () => HTMLElement | null,
  ms: number,
): Promise<void> {
  setVisible(false);
  await waitPanelTransition(getEl(), ms);
  setMounted(false);
}
