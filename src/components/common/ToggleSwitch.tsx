interface Props {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  disabled?: boolean;
  id?: string;
  ariaLabel: string;
}

export default function ToggleSwitch({
  checked,
  onCheckedChange,
  disabled = false,
  id,
  ariaLabel,
}: Props) {
  return (
    <label
      htmlFor={id}
      className={`toggle-switch${disabled ? " toggle-switch--disabled" : ""}`}
    >
      <input
        id={id}
        type="checkbox"
        className="toggle-switch-input"
        checked={checked}
        disabled={disabled}
        onChange={(e) => onCheckedChange(e.target.checked)}
        aria-label={ariaLabel}
      />
      <span className="toggle-switch-track" aria-hidden="true">
        <span className="toggle-switch-knob" />
      </span>
    </label>
  );
}
