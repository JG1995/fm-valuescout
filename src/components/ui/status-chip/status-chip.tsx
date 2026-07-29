import type { LucideIcon } from "lucide-react";
import { cn } from "@/utils/cn";

type StatusChipTone = "success" | "warning" | "error" | "info" | "neutral";

type StatusChipProps = {
  tone: StatusChipTone;
  /** Required: a chip must never carry its meaning in fill colour alone. */
  icon: LucideIcon;
  children: string;
  className?: string;
};

// Container fills sit only ~1.4:1 above the panel, so each carries a 40%-alpha
// border in its own tone to give the pill a readable edge.
const toneClasses: Record<StatusChipTone, string> = {
  success: "bg-success-container text-on-success-container border-success/40",
  warning: "bg-warning-container text-on-warning-container border-warning/40",
  error: "bg-error-container text-on-error-container border-error/40",
  info: "bg-info-container text-on-info-container border-info/40",
  neutral:
    "bg-surface-container-high text-on-surface-variant border-outline-variant",
};

export function StatusChip({
  tone,
  icon: Icon,
  children,
  className,
}: StatusChipProps) {
  return (
    <span
      className={cn(
        "inline-flex h-5 items-center gap-1.5 rounded-full border px-2 text-label-md",
        toneClasses[tone],
        className,
      )}
    >
      <Icon
        aria-hidden="true"
        size={12}
        strokeWidth={1.5}
        className="shrink-0"
      />
      {children}
    </span>
  );
}
