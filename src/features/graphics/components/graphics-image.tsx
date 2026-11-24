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

// Chromium keeps an image in memory only while an element still references it.
// Holding recently loaded images lets a remounted virtualized row reuse them
// without a new protocol request; the responses stay `no-store`, so nothing
// outlives the page.
const RETAINED_IMAGE_LIMIT = 256;
const retainedImages = new Map<string, HTMLImageElement>();

function retainLoadedImage(source: string) {
  let image = retainedImages.get(source);
  if (image) {
    retainedImages.delete(source);
  } else {
    image = new Image();
    image.src = source;
  }
  retainedImages.set(source, image);
  if (retainedImages.size > RETAINED_IMAGE_LIMIT) {
    retainedImages.delete(retainedImages.keys().next().value as string);
  }
}

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
      onLoad={() => retainLoadedImage(source)}
      onError={() => setFailedSource(source)}
    />
  );
}
