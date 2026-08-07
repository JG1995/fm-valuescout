import { queryOptions } from "@tanstack/react-query";
import { academyKeys } from "./academy-keys";
import { fetchAcademyClass } from "./fetch-academy-class";

export function academyClassQueryOptions(classId: number) {
  return queryOptions({
    queryKey: academyKeys.academyClass(classId),
    queryFn: () => fetchAcademyClass(classId),
  });
}
