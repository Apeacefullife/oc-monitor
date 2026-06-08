import { useCallback, useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  resolveCursorVariant,
  type CursorVariant,
} from "../../utils/cursorTarget";
import { queryCursorClientPoint } from "../../utils/cursorWindowBounds";

const CURSORS = {
  arrow: { src: "/cursors/pointer-32.png", hotX: 7, hotY: 4, size: 32 },
  hand: { src: "/cursors/hand-32.png", hotX: 12, hotY: 3, size: 32 },
} as const satisfies Record<
  CursorVariant,
  { src: string; hotX: number; hotY: number; size: number }
>;

interface CursorState {
  visible: boolean;
  x: number;
  y: number;
  variant: CursorVariant;
}

function isInsideViewport(x: number, y: number) {
  const margin = 2;
  return (
    x >= -margin &&
    y >= -margin &&
    x <= window.innerWidth + margin &&
    y <= window.innerHeight + margin
  );
}

export default function CustomCursor() {
  const [state, setState] = useState<CursorState>({
    visible: false,
    x: 0,
    y: 0,
    variant: "arrow",
  });
  const rafRef = useRef(0);
  const pollRef = useRef(0);
  const visibleRef = useRef(false);
  const checkingRef = useRef(false);
  const pendingRef = useRef<{
    x: number;
    y: number;
    target: EventTarget | null;
  } | null>(null);

  const hideCursor = useCallback(() => {
    pendingRef.current = null;
    visibleRef.current = false;
    if (rafRef.current) {
      cancelAnimationFrame(rafRef.current);
      rafRef.current = 0;
    }
    setState((prev) => (prev.visible ? { ...prev, visible: false } : prev));
  }, []);

  const showCursor = useCallback(
    (x: number, y: number, target: EventTarget | null) => {
      visibleRef.current = true;
      setState({
        visible: true,
        x,
        y,
        variant: resolveCursorVariant(target),
      });
    },
    [],
  );

  useEffect(() => {
    visibleRef.current = state.visible;
  }, [state.visible]);

  useEffect(() => {
    document.documentElement.classList.add("use-custom-cursor");

    const verifyGlobalBounds = async () => {
      if (checkingRef.current || !visibleRef.current) return;
      checkingRef.current = true;
      try {
        const point = await queryCursorClientPoint();
        if (!point) return;
        if (!point.inside || !isInsideViewport(point.x, point.y)) {
          hideCursor();
          return;
        }
        setState((prev) =>
          prev.visible
            ? { ...prev, x: point.x, y: point.y }
            : prev,
        );
      } finally {
        checkingRef.current = false;
      }
    };

    const flush = () => {
      rafRef.current = 0;
      const pending = pendingRef.current;
      if (!pending) return;
      pendingRef.current = null;

      if (!isInsideViewport(pending.x, pending.y)) {
        hideCursor();
        return;
      }

      showCursor(pending.x, pending.y, pending.target);
      void verifyGlobalBounds();
    };

    const schedule = (x: number, y: number, target: EventTarget | null) => {
      if (!isInsideViewport(x, y)) {
        hideCursor();
        return;
      }

      pendingRef.current = { x, y, target };
      if (!rafRef.current) {
        rafRef.current = requestAnimationFrame(flush);
      }
    };

    const onMove = (event: MouseEvent) => {
      schedule(event.clientX, event.clientY, event.target);
    };

    const onOut = (event: MouseEvent) => {
      const next = event.relatedTarget;
      if (next && next instanceof Node && document.documentElement.contains(next)) {
        return;
      }
      hideCursor();
      void verifyGlobalBounds();
    };

    const onBlur = () => {
      hideCursor();
    };

    const poll = () => {
      if (visibleRef.current) {
        void verifyGlobalBounds();
      }
      pollRef.current = requestAnimationFrame(poll);
    };
    pollRef.current = requestAnimationFrame(poll);

    window.addEventListener("mousemove", onMove, {
      capture: true,
      passive: true,
    });
    document.documentElement.addEventListener("mouseout", onOut, {
      capture: true,
    });
    window.addEventListener("blur", onBlur);
    window.addEventListener("ds-cursor-hide", hideCursor);

    const cleanups: Array<() => void> = [];

    void listen("window-mouse-leave", hideCursor).then((unlisten) => {
      cleanups.push(unlisten);
    });

    void getCurrentWindow()
      .setCursorVisible(false)
      .catch(() => {
        // 非 Tauri 环境忽略
      });

    void getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (!focused) hideCursor();
      })
      .then((unlisten) => {
        cleanups.push(unlisten);
      });

    return () => {
      document.documentElement.classList.remove("use-custom-cursor");
      window.removeEventListener("mousemove", onMove, { capture: true });
      document.documentElement.removeEventListener("mouseout", onOut, {
        capture: true,
      });
      window.removeEventListener("blur", onBlur);
      window.removeEventListener("ds-cursor-hide", hideCursor);
      cleanups.forEach((fn) => fn());
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      if (pollRef.current) cancelAnimationFrame(pollRef.current);
    };
  }, [hideCursor, showCursor]);

  if (!state.visible) return null;

  const cfg = CURSORS[state.variant];

  return createPortal(
    <img
      src={cfg.src}
      alt=""
      aria-hidden
      draggable={false}
      className="ds-custom-cursor"
      style={{
        transform: `translate3d(${state.x - cfg.hotX}px, ${state.y - cfg.hotY}px, 0)`,
        width: cfg.size,
        height: cfg.size,
      }}
    />,
    document.body,
  );
}
