import { queryOptions } from "@tanstack/react-query";
import { academyKeys } from "./academy-keys";
import { fetchAcademyClasses } from "./fetch-academy-classes";

export const academyClassesQueryOptions = queryOptions({
  queryKey: academyKeys.classes(),
  queryFn: fetchAcademyClasses,
});
