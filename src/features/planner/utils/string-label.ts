/** "1st", "2nd", and so on for a 0-based position. */
export function ordinal(position: number): string {
  const number = position + 1;
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
  return `${number}${suffix}`;
}

export function ordinalStringLabel(stringOrder: number): string {
  return `${ordinal(stringOrder)} string`;
}

export function nextOrdinalDefault(existingCount: number): string {
  return ordinalStringLabel(existingCount);
}
