import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "@/utils/cn";

type ButtonVariant = "primary" | "secondary";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  children: ReactNode;
  variant?: ButtonVariant;
};

export function Button({
  children,
  className,
  variant = "primary",
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cn(
        "rounded px-3 py-1.5 text-sm",
        variant === "primary" && "bg-primary text-on-primary",
        variant === "secondary" &&
          "border border-outline bg-surface text-on-surface",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
