import { queryOptions } from "@tanstack/react-query";
import { fetchDemoValue } from "./fetch-demo-value";
import { healthKeys } from "./health-keys";

export const demoValueQueryOptions = queryOptions({
  queryKey: healthKeys.demoValue(),
  queryFn: fetchDemoValue,
});
