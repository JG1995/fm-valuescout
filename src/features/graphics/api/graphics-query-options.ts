import { queryOptions } from "@tanstack/react-query";

import { fetchGraphicsStatus } from "./fetch-graphics-status";
import { graphicsKeys } from "./graphics-keys";

export const graphicsStatusQueryOptions = queryOptions({
  queryKey: graphicsKeys.status(),
  queryFn: fetchGraphicsStatus,
});
