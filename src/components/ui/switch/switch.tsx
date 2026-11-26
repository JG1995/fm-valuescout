import type { InputHTMLAttributes } from "react";
import { cn } from "@/utils/cn";

type SwitchProps = Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "type" | "role" | "className" | "children" | "checked" | "onChange"
> & {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  className?: string;
};

/**
 * An on/off setting that takes effect at once. A native checkbox carries the
 * state, the keyboard, and the label; the track and thumb are drawn over it.
 */
export function Switch({
  label,
  checked,
  onChange,
  disabled,
  className,
  ...props
}: SwitchProps) {
  return (
    <label
      className={cn(
        "inline-flex h-8 items-center gap-2 text-body-md",
        disabled ? "text-on-surface-variant" : "text-on-surface",
        className,
      )}
    >
      <span className="relative inline-flex h-5 w-9 shrink-0">
        <input
          type="checkbox"
          // biome-ignore lint/a11y/useAriaPropsForRole: a native checkbox reports its own checked state.
          role="switch"
          checked={checked}
          disabled={disabled}
          className="peer absolute inset-0 m-0 size-full opacity-0"
          onChange={(event) => onChange(event.target.checked)}
          {...props}
        />
        <span
          aria-hidden="true"
          className={cn(
            "pointer-events-none size-full rounded-md border border-outline bg-surface-container-high",
            "transition-colors duration-150 ease-out",
            // The thumb: a square that slides, so the state reads without colour.
            "after:absolute after:top-0.5 after:left-0.5 after:size-3.5 after:rounded-sm after:bg-on-surface-variant",
            "after:transition-[translate,background-color] after:duration-150 after:ease-out",
            "peer-hover:border-on-surface-variant",
            "peer-checked:border-primary peer-checked:bg-primary peer-checked:after:translate-x-4 peer-checked:after:bg-on-primary",
            "peer-focus-visible:outline-2 peer-focus-visible:outline-offset-2 peer-focus-visible:outline-primary",
            "peer-disabled:opacity-45",
          )}
        />
      </span>
      {label}
    </label>
  );
}
