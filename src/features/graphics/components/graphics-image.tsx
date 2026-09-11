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
  const [failed, setFailed] = useState(false);
  const generation = status?.generation;

  if (
    !status?.selected ||
    generation === undefined ||
    !Number.isInteger(uid) ||
    uid <= 0 ||
    failed
  ) {
    return slot.fallback ?? null;
  }

  return (
    <img
      src={`http://graphics.localhost/${generation}/${kind}/${uid}`}
      alt={slot.alt}
      aria-hidden={slot.ariaHidden}
      className={slot.className}
      loading="lazy"
      decoding="async"
      onError={() => setFailed(true)}
    />
  );
}
