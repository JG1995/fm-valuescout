import type { LucideIcon } from "lucide-react";
import type { ReactNode } from "react";
import { cn } from "@/utils/cn";

type EmptyStateProps = {
  icon: LucideIcon;
  /** Names the situation in the user's terms — never a raw error string. */
  title: string;
  /** One line explaining the state, or the underlying reason for a failure. */
  children: ReactNode;
  /** The single next step. A state with no way forward is a dead end. */
  action?: ReactNode;
  className?: string;
};

/** The Empty, Loading, and Error states of `.wiki/DESIGN.md`: every data view
 *  defines all three, and a blank region is a bug. The tone stays neutral even
 *  for failures — semantic colour belongs to chips and banners. */
export function EmptyState({
  icon: Icon,
  title,
  children,
  action,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex min-h-40 flex-col items-center justify-center gap-2 p-6 text-center",
        className,
      )}
    >
      <Icon
        aria-hidden="true"
        size={24}
        strokeWidth={1.5}
        className="shrink-0 text-on-surface-variant"
      />
      <p className="text-headline-sm text-on-surface">{title}</p>
      <p className="max-w-prose text-body-md text-on-surface-variant">
        {children}
      </p>
      {action}
    </div>
  );
}
