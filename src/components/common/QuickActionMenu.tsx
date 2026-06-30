import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useT } from "../../i18n";

export interface QuickActionMenuState {
  x: number;
  y: number;
}

interface Action {
  id: string;
  label: string;
  onClick: () => void;
}

interface Props {
  state: QuickActionMenuState | null;
  onClose: () => void;
  onRefresh: () => void;
  onOpenAnalysis: () => void;
  onOpenSettings: () => void;
  onHideToTray: () => void;
}

const MARGIN = 8;
const OFFSET = 4;

function computePosition(
  anchorX: number,
  anchorY: number,
  width: number,
  height: number,
) {
  const vw = window.innerWidth;
  const vh = window.innerHeight;

  let left = anchorX + OFFSET;
  let top = anchorY + OFFSET;

  if (left + width > vw - MARGIN) {
    left = anchorX - width - OFFSET;
  }
  if (top + height > vh - MARGIN) {
    top = anchorY - height - OFFSET;
  }

  left = Math.max(MARGIN, Math.min(left, vw - width - MARGIN));
  top = Math.max(MARGIN, Math.min(top, vh - height - MARGIN));

  return { left, top };
}

export default function QuickActionMenu({
  state,
  onClose,
  onRefresh,
  onOpenAnalysis,
  onOpenSettings,
  onHideToTray,
}: Props) {
  const t = useT();
  const panelRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<{ left: number; top: number } | null>(
    null,
  );

  const actions: Action[] = [
    {
      id: "refresh",
      label: t("quickAction.refresh"),
      onClick: onRefresh,
    },
    {
      id: "analysis",
      label: t("quickAction.analysis"),
      onClick: onOpenAnalysis,
    },
    {
      id: "settings",
      label: t("quickAction.settings"),
      onClick: onOpenSettings,
    },
    {
      id: "hide",
      label: t("quickAction.hideToTray"),
      onClick: onHideToTray,
    },
  ];

  useLayoutEffect(() => {
    if (!state || !panelRef.current) {
      setPosition(null);
      return;
    }

    const { offsetWidth, offsetHeight } = panelRef.current;
    setPosition(computePosition(state.x, state.y, offsetWidth, offsetHeight));
  }, [state]);

  useEffect(() => {
    if (!state) return;

    const handleKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    const handlePointer = (event: MouseEvent) => {
      if (panelRef.current?.contains(event.target as Node)) return;
      onClose();
    };

    window.addEventListener("keydown", handleKey);
    window.addEventListener("mousedown", handlePointer);

    return () => {
      window.removeEventListener("keydown", handleKey);
      window.removeEventListener("mousedown", handlePointer);
    };
  }, [state, onClose]);

  if (!state) return null;

  return createPortal(
    <div
      className="quick-action-menu-layer"
      style={{
        left: position?.left ?? state.x + OFFSET,
        top: position?.top ?? state.y + OFFSET,
        visibility: position ? "visible" : "hidden",
      }}
      onMouseDown={(event) => event.stopPropagation()}
    >
      <div ref={panelRef} className="quick-action-menu" role="menu">
        {actions.map((action, index) => (
          <div key={action.id}>
            {index === actions.length - 1 ? (
              <div className="quick-action-menu-divider" />
            ) : null}
            <button
              type="button"
              role="menuitem"
              className="quick-action-menu-item"
              onClick={() => {
                action.onClick();
                onClose();
              }}
            >
              {action.label}
            </button>
          </div>
        ))}
      </div>
    </div>,
    document.body,
  );
}
