/**
 * Shared presentation formatters. Every displayed value goes through here so
 * two screens cannot render the same fact differently — see the "Value & Number
 * Formatting" section of `.wiki/DESIGN.md`.
 */

const MINUTE_MS = 60_000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;

/**
 * Relative snapshot age, for the freshness chip and any "loaded" line. Relative
 * age is what tells the user whether to reload; the absolute stamp belongs in a
 * `title` attribute via {@link formatAbsoluteUtc}.
 */
export function formatRelativeAge(isoUtc: string, now = Date.now()): string {
  const then = Date.parse(isoUtc);
  if (Number.isNaN(then)) {
    return "unknown";
  }

  const elapsed = now - then;
  if (elapsed < MINUTE_MS) {
    return "just now";
  }
  if (elapsed < HOUR_MS) {
    const minutes = Math.floor(elapsed / MINUTE_MS);
    return `${minutes} min ago`;
  }
  if (elapsed < DAY_MS) {
    const hours = Math.floor(elapsed / HOUR_MS);
    return `${hours} ${hours === 1 ? "hour" : "hours"} ago`;
  }
  const days = Math.floor(elapsed / DAY_MS);
  if (days === 1) {
    return "yesterday";
  }
  return `${days} days ago`;
}

/** Absolute UTC stamp for `title` attributes: `2026-07-29 20:14 UTC`. */
export function formatAbsoluteUtc(isoUtc: string): string {
  const parsed = new Date(isoUtc);
  if (Number.isNaN(parsed.getTime())) {
    return isoUtc;
  }
  return `${parsed.toISOString().slice(0, 16).replace("T", " ")} UTC`;
}

/** Integer counts with thousands separators: `1,247`. */
export function formatCount(value: number): string {
  return value.toLocaleString("en-GB");
}

/** Absent data and zero are different facts, so missing values render as an em dash. */
export function formatMissable(value: string | number | null | undefined) {
  return value === null || value === undefined || value === "" ? "—" : value;
}

/**
 * Money per DESIGN.md — euro prefix, abbreviated by magnitude. Source values
 * are GBP from FM; display currency follows the design system.
 */
export function formatMoney(value: number): string {
  if (value < 1_000) {
    return `€${value}`;
  }
  if (value < 1_000_000) {
    return `€${Math.round(value / 1_000)}k`;
  }
  const millions = value / 1_000_000;
  if (value < 100_000_000) {
    const rounded = Math.round(millions * 10) / 10;
    return Number.isInteger(rounded)
      ? `€${rounded}M`
      : `€${rounded.toFixed(1)}M`;
  }
  return `€${Math.round(millions)}M`;
}

/** Birth date from FM day-of-year, with optional age: `21/03/2001 (25)`. */
export function formatPlayerDob(
  birthYear: number,
  birthDayOfYear: number,
  age: number | null,
): string {
  const date = new Date(Date.UTC(birthYear, 0, birthDayOfYear));
  const day = String(date.getUTCDate()).padStart(2, "0");
  const month = String(date.getUTCMonth() + 1).padStart(2, "0");
  const dob = `${day}/${month}/${birthYear}`;
  return age === null ? dob : `${dob} (${age})`;
}
