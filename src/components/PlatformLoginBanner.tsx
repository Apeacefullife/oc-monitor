import { useT } from "../i18n";

export type PlatformBannerMode = "required" | "waiting" | "expired";

interface Props {
  mode: PlatformBannerMode;
  onLogin: () => void;
}

export default function PlatformLoginBanner({ mode, onLogin }: Props) {
  const t = useT();

  const isExpired = mode === "expired";
  const title = isExpired ? t("platform.expired") : t("platform.why");
  const hint =
    mode === "waiting"
      ? t("platform.waiting")
      : isExpired
        ? null
        : t("platform.persist");

  return (
    <div
      className={`mx-4 mb-2 px-3 py-2.5 rounded-xl text-xs shrink-0 border ${
        isExpired
          ? "bg-amber-500/10 border-amber-500/25 text-amber-200/90"
          : "bg-deepseek-500/10 border-deepseek-500/20 text-white/70"
      }`}
    >
      <p className="leading-relaxed">{title}</p>
      {hint && (
        <p className="mt-1 text-[10px] text-white/45 leading-relaxed">{hint}</p>
      )}
      {mode !== "waiting" && (
        <button
          type="button"
          onClick={onLogin}
          className={`mt-2 px-3 py-1 rounded-lg text-[11px] transition-colors ${
            isExpired
              ? "bg-amber-500/20 text-amber-300 hover:bg-amber-500/30"
              : "bg-deepseek-500/20 text-deepseek-400 hover:bg-deepseek-500/30"
          }`}
        >
          {isExpired ? t("platform.reloginBtn") : t("platform.loginBtn")}
        </button>
      )}
    </div>
  );
}
