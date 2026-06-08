import { useT } from "../../i18n";
import { formatRelativeTime } from "../../utils/format";

interface Props {
  loading: boolean;
  lastUpdated: string | null;
  status?: string | null;
}

export default function StatusIndicator({
  loading,
  lastUpdated,
  status,
}: Props) {
  const t = useT();

  const getStatusColor = () => {
    switch (status) {
      case "normal":
        return "bg-success";
      case "low":
        return "bg-warning";
      case "exhausted":
        return "bg-danger";
      default:
        return "bg-gray-500";
    }
  };

  return (
    <div className="flex items-center gap-2 text-xs ui-muted">
      {loading && (
        <span className="text-deepseek-400 text-[10px] animate-pulse">
          {t("status.updating")}
        </span>
      )}
      {lastUpdated && !loading && (
        <span className="text-[10px]">{formatRelativeTime(lastUpdated)}</span>
      )}
      <div className={`w-1.5 h-1.5 rounded-full ${getStatusColor()} ${loading ? "animate-pulse" : ""}`} />
    </div>
  );
}
