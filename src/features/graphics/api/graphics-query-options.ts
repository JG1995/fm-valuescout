import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { GraphicsKind, GraphicsResult } from "../types/graphics";
import { fetchGraphicsStatus } from "./fetch-graphics-status";
import { graphicsKeys } from "./graphics-keys";

export const graphicsStatusQueryOptions = queryOptions({
  queryKey: graphicsKeys.status(),
  queryFn: fetchGraphicsStatus,
});

export function graphicsResultQueryOptions(
  generation: number,
  kind: GraphicsKind,
  uid: number,
) {
  return queryOptions({
    queryKey: graphicsKeys.result(generation, kind, uid),
    queryFn: () =>
      invokeCommand<GraphicsResult>("resolve_graphics", { kind, uid }),
    enabled: uid > 0,
    placeholderData: undefined,
  });
}
