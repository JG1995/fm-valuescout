import type { ReactNode, SelectHTMLAttributes } from "react";
import { useId } from "react";
import {
  fieldClasses,
  fieldLabelClasses,
} from "@/components/ui/field/field-styles";
import { cn } from "@/utils/cn";

type SelectFieldProps = Omit<
  SelectHTMLAttributes<HTMLSelectElement>,
  "id" | "className"
> & {
  label: string;
  children: ReactNode;
  className?: string;
};

// Native select: `color-scheme: dark` in the base layer already makes the popup
// list match the app, so a custom listbox would only reimplement it.
export function SelectField({
  label,
  children,
  className,
  ...props
}: SelectFieldProps) {
  const selectId = useId();

  return (
    <div className={cn("space-y-1", className)}>
      <label className={fieldLabelClasses} htmlFor={selectId}>
        {label}
      </label>
      <select id={selectId} className={cn(fieldClasses, "w-full")} {...props}>
        {children}
      </select>
    </div>
  );
}
