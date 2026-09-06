export function ordinalStringLabel(stringOrder: number): string {
  const number = stringOrder + 1;
  const suffix =
    number % 100 >= 11 && number % 100 <= 13
      ? "th"
      : number % 10 === 1
        ? "st"
        : number % 10 === 2
          ? "nd"
          : number % 10 === 3
            ? "rd"
            : "th";
  return `${number}${suffix} string`;
}

export function nextOrdinalDefault(existingCount: number): string {
  return ordinalStringLabel(existingCount);
}
