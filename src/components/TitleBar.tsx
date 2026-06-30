import { type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useT } from "../i18n";

interface Props {
  hidden?: boolean;
}

function TitleBarButton({
  onClick,
  title,
  dangerHover = false,
  children,
}: {
  onClick: () => void;
  title: string;
  dangerHover?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`w-6 h-6 flex items-center justify-center rounded-md transition-all duration-200 ${
        dangerHover
          ? "text-white/25 hover:text-danger/70 hover:bg-danger/10"
          : "text-white/25 hover:text-white/50 hover:bg-white/5"
      }`}
      title={title}
    >
      {children}
    </button>
  );
}

export default function TitleBar({ hidden = false }: Props) {
  const t = useT();
  const appWindow = getCurrentWindow();

  const handleMinimize = async () => {
    try {
      await appWindow.minimize();
    } catch (err) {
      console.error("minimize failed:", err);
    }
  };

  const handleHide = async () => {
    try {
      await appWindow.hide();
    } catch (err) {
      console.error("hide failed:", err);
    }
  };

  return (
    <div
      className={`relative z-50 flex items-center h-9 select-none shrink-0 titlebar ${hidden ? "invisible" : ""}`}
      onMouseLeave={(event) => {
        const next = event.relatedTarget;
        if (
          event.clientY <= 0 ||
          !next ||
          !(next instanceof Node) ||
          !document.documentElement.contains(next)
        ) {
          window.dispatchEvent(new Event("ds-cursor-hide"));
        }
      }}
    >
      <div
        data-tauri-drag-region
        onDoubleClick={(e) => e.preventDefault()}
        className="titlebar-drag flex flex-1 items-center gap-2 pl-4 pr-3 h-full min-w-0"
      >
        <div className="h-4 w-5 shrink-0 flex items-center justify-center pointer-events-none">
          <svg
            className="h-4 w-auto"
            viewBox="0 0 1391 1024"
            fill="#4D6BFE"
            aria-hidden
          >
            <path d="M950.6 231.9c-9.5-4.7-13.6 4.3-19.1 8.9-1.9 1.5-3.5 3.4-5.1 5.2-13.9 15.1-30.1 25.1-51.2 23.9-31-1.8-57.4 8.2-80.7 32.3-5-29.8-21.5-47.6-46.6-59-13.2-5.9-26.4-11.9-35.6-24.8-6.4-9.2-8.2-19.4-11.4-29.5-2-6.1-4.1-12.3-10.9-13.3-7.5-1.2-10.4 5.2-13.3 10.5-11.7 21.8-16.2 45.8-15.8 70.1 1 54.7 23.7 98.3 68.6 129.3 5.1 3.6 6.4 7.1 4.8 12.3-3.1 10.7-6.7 21.1-9.9 31.7-2 6.8-5.1 8.3-12.3 5.3-24.7-10.5-46-26.1-64.8-44.9-32-31.6-60.9-66.4-96.9-93.7-8.3-6.3-16.9-12.3-25.7-17.9-36.8-36.5 4.8-66.4 14.5-70 10.1-3.7 3.5-16.5-29.1-16.3-32.6 0.1-62.3 11.3-100.3 26.1-5.6 2.2-11.4 3.9-17.4 5.2-34.5-6.7-70.2-8.2-107.6-3.9-70.4 8-126.6 42-167.9 99.9-49.6 69.7-61.3 148.9-47 231.5 15 87 58.5 159.1 125.4 215.4 69.4 58.4 149.2 87 240.3 81.6 55.3-3.3 117-10.8 186.5-70.9 17.5 8.9 35.9 12.5 66.4 15.1 23.5 2.2 46.2-1.2 63.7-4.9 27.4-5.9 25.5-31.9 15.6-36.6-80.5-38.3-62.8-22.7-78.8-35.3 40.9-49.4 102.5-100.7 126.6-266.9 1.9-13.2 0.3-21.5 0-32.2-0.1-6.5 1.3-9 8.6-9.8 20.1-2.4 39.7-8 57.7-18.1 52.1-29.1 73.2-76.8 78.1-134 0.5-8.6-0.4-17.7-9.4-22.3M496.3 747.1c-78-62.6-115.8-83.2-131.4-82.3-14.6 0.9-12 17.9-8.8 29.1 3.4 11 7.7 18.5 13.9 28.2 4.2 6.4 7.2 15.9-4.2 23-25.1 15.9-68.8-5.3-70.8-6.4-50.8-30.5-93.3-70.9-123.3-126-28.9-53.1-45.7-110-48.5-170.8-0.7-14.7 3.5-19.9 17.8-22.5 18.8-3.7 38.1-4.2 57.1-1.5 79.6 11.9 147.3 48.2 204.1 105.7 32.4 32.8 56.9 71.9 82.2 110.2 26.9 40.6 55.8 79.3 92.6 111.1 13 11.1 23.4 19.6 33.3 25.8-29.9 3.2-79.8 3.9-114-23.6m37.4-245.4c0-6.5 5.1-11.7 11.5-11.7 1.5 0 2.8 0.3 3.9 0.7 4.5 1.7 7.5 6.1 7.5 11 0.1 6.4-5 11.6-11.4 11.7h-0.1c-6.3 0-11.4-5.2-11.4-11.5v-0.2m116.1 60.7c-7.4 3.1-14.9 5.8-22.1 6.1-11.1 0.6-23.2-4-29.8-9.6-10.2-8.7-17.5-13.6-20.6-28.9-1.3-6.5-0.6-16.6 0.6-22.4 2.6-12.5-0.3-20.5-8.9-27.7-7-5.9-15.9-7.6-25.7-7.6-3.7 0-7-1.6-9.5-3-4.8-2.3-6.9-8.1-4.6-13 0.1-0.2 0.2-0.4 0.3-0.7 1-2.1 6-7.1 7.2-8 13.3-7.7 28.6-5.2 42.8 0.6 13.1 5.5 23.1 15.6 37.4 29.8 14.6 17.2 17.2 21.9 25.5 34.8 6.6 10.1 12.6 20.5 16.6 32.3 2.6 7.6-0.6 13.6-9.2 17.3" />
          </svg>
        </div>
        <span className="text-[11px] font-medium text-white/40 tracking-wide pointer-events-none">
          {t("app.name")}
        </span>
      </div>

      <div
        data-tauri-drag-region="false"
        className="titlebar-actions flex items-center gap-0.5 pr-3 shrink-0"
      >
        <TitleBarButton
          onClick={() => void handleMinimize()}
          title={t("titlebar.minimize")}
        >
          <svg
            className="w-3 h-3 pointer-events-none"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path strokeLinecap="round" strokeLinejoin="round" d="M20 12H4" />
          </svg>
        </TitleBarButton>

        <TitleBarButton
          onClick={() => void handleHide()}
          title={t("titlebar.hideToTray")}
          dangerHover
        >
          <svg
            className="w-3 h-3 pointer-events-none"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </TitleBarButton>
      </div>
    </div>
  );
}
