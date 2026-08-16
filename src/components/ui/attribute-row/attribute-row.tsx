import type { ReactNode } from "react";

export function AttributeRow({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="flex min-h-9 min-w-0 items-center justify-between gap-3 border-b border-outline-variant/70">
      <dt className="truncate text-body-md text-on-surface-variant">{label}</dt>
      <dd className="shrink-0 font-mono text-mono-sm tabular-nums">
        {children}
      </dd>
    </div>
  );
}
