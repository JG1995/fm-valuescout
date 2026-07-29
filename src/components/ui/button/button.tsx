import type { LucideIcon } from "lucide-react";
import { LoaderCircle } from "lucide-react";
import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "@/utils/cn";

type ButtonVariant = "primary" | "secondary" | "ghost";
type ButtonSize = "default" | "lg" | "icon";

type ButtonBaseProps = Omit<
  ButtonHTMLAttributes<HTMLButtonElement>,
  "children"
> & {
  variant?: ButtonVariant;
  icon?: LucideIcon;
  loading?: boolean;
  /** Phase-specific label shown while `loading` (for example "Scanning…"). */
  loadingLabel?: string;
};

/** An icon-only button has no visible text, so the type demands an icon and an
 *  accessible name instead of trusting a review pass to notice. */
type ButtonProps = ButtonBaseProps &
  (
    | { size?: "default" | "lg"; children: ReactNode }
    | {
        size: "icon";
        icon: LucideIcon;
        "aria-label": string;
        children?: never;
      }
  );

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    "bg-primary text-on-primary hover:bg-primary-hover active:bg-primary-active",
  secondary:
    "border border-outline text-on-surface hover:bg-surface-container-high active:bg-surface-container-highest",
  ghost:
    "text-on-surface-variant hover:bg-surface-container-high hover:text-on-surface active:bg-surface-container-highest",
};

const sizeClasses: Record<ButtonSize, string> = {
  default: "h-8 rounded-full px-4",
  lg: "h-9 rounded-full px-4",
  icon: "size-8 rounded-md",
};

export function Button({
  children,
  className,
  variant = "primary",
  size = "default",
  icon: Icon,
  loading = false,
  loadingLabel,
  type = "button",
  disabled,
  ...props
}: ButtonProps) {
  const LeadingIcon = loading ? LoaderCircle : Icon;

  return (
    <button
      type={type}
      disabled={disabled || loading}
      className={cn(
        "inline-flex cursor-pointer items-center justify-center gap-2 text-label-lg",
        "transition-colors duration-150 ease-out",
        "disabled:cursor-not-allowed disabled:opacity-45",
        variantClasses[variant],
        sizeClasses[size],
        className,
      )}
      {...props}
    >
      {LeadingIcon ? (
        <LeadingIcon
          aria-hidden="true"
          size={16}
          strokeWidth={1.5}
          className={cn("shrink-0", loading && "animate-spin")}
        />
      ) : null}
      {children ? (
        // Both labels share one grid cell so the button keeps its widest width
        // across states — Load Data must not jump when it becomes "Scanning…".
        // The inactive one is aria-hidden as well as invisible, so its text
        // stays out of the accessible name even where the stylesheet is absent.
        <span className="grid">
          <span
            aria-hidden={loading || undefined}
            className={cn("col-start-1 row-start-1", loading && "invisible")}
          >
            {children}
          </span>
          {loadingLabel ? (
            <span
              aria-hidden={!loading || undefined}
              className={cn("col-start-1 row-start-1", !loading && "invisible")}
            >
              {loadingLabel}
            </span>
          ) : null}
        </span>
      ) : null}
    </button>
  );
}
