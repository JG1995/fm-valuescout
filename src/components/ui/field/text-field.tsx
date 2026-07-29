import type { InputHTMLAttributes } from "react";
import { useId } from "react";
import {
  fieldClasses,
  fieldLabelClasses,
} from "@/components/ui/field/field-styles";
import { cn } from "@/utils/cn";

type TextFieldProps = Omit<
  InputHTMLAttributes<HTMLInputElement>,
  "id" | "className"
> & {
  label: string;
  /** Shown below the field and announced with it. */
  error?: string;
  className?: string;
};

export function TextField({
  label,
  error,
  className,
  ...props
}: TextFieldProps) {
  const inputId = useId();
  const errorId = useId();

  return (
    <div className={cn("space-y-1", className)}>
      <label className={fieldLabelClasses} htmlFor={inputId}>
        {label}
      </label>
      <input
        id={inputId}
        aria-describedby={error ? errorId : undefined}
        aria-invalid={error ? true : undefined}
        className={cn(fieldClasses, "w-full", error && "border-error")}
        {...props}
      />
      {error ? (
        <p className="text-body-sm text-error" id={errorId}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
