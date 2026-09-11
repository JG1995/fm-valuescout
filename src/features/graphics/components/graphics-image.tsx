import { useQuery } from "@tanstack/react-query";
import { type ReactNode, useState } from "react";
import { graphicsStatusQueryOptions } from "../api/graphics-query-options";
import type { GraphicsKind } from "../types/graphics";

export type GraphicsImageSlot = {
  alt: string;
  className: string;
  ariaHidden?: boolean;
  fallback?: ReactNode;
};

type GraphicsImageProps = {
  kind: GraphicsKind;
  uid: number;
  slot: GraphicsImageSlot;
};

export function GraphicsImage({ kind, uid, slot }: GraphicsImageProps) {
  const { data: status } = useQuery(graphicsStatusQueryOptions);
  const [failedSource, setFailedSource] = useState<string | null>(null);
  const generation = status?.generation;
  const source = `http://graphics.localhost/${generation}/${kind}/${uid}`;

  if (
    !status?.selected ||
    status.rebuilding ||
    generation === undefined ||
    !Number.isInteger(uid) ||
    uid <= 0 ||
    failedSource === source
  ) {
    return slot.fallback ?? null;
  }

  return (
    <img
      src={source}
      alt={slot.alt}
      aria-hidden={slot.ariaHidden}
      className={slot.className}
      loading="lazy"
      decoding="async"
      onError={() => setFailedSource(source)}
    />
  );
}
