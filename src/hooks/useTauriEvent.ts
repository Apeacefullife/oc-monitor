import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Tauri 事件监听 Hook
 * @param event 事件名
 * @param handler 事件处理器
 */
export function useTauriEvent<T = unknown>(
  event: string,
  handler: (payload: T) => void,
) {
  const savedHandler = useRef(handler);

  useEffect(() => {
    savedHandler.current = handler;
  }, [handler]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;

    const setup = async () => {
      unlisten = await listen<T>(event, (e) => {
        savedHandler.current(e.payload);
      });
    };

    setup();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [event]);
}
