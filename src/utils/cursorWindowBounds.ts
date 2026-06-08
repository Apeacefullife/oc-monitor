import { cursorPosition, getCurrentWindow } from "@tauri-apps/api/window";

export interface CursorClientPoint {
  x: number;
  y: number;
  inside: boolean;
}

/** 用屏幕坐标判断鼠标是否仍在窗口内（不依赖 WebView 的 mouseleave） */
export async function queryCursorClientPoint(): Promise<CursorClientPoint | null> {
  try {
    const win = getCurrentWindow();
    if (!(await win.isVisible())) {
      return { x: 0, y: 0, inside: false };
    }

    const [outerPos, outerSize, scale, global] = await Promise.all([
      win.outerPosition(),
      win.outerSize(),
      win.scaleFactor(),
      cursorPosition(),
    ]);

    const inside =
      global.x >= outerPos.x &&
      global.y >= outerPos.y &&
      global.x < outerPos.x + outerSize.width &&
      global.y < outerPos.y + outerSize.height;

    return {
      x: (global.x - outerPos.x) / scale,
      y: (global.y - outerPos.y) / scale,
      inside,
    };
  } catch {
    return null;
  }
}
