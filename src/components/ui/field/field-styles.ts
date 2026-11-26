/** Shared control chrome for every form field, so a text input and a select
 *  cannot drift apart visually. Width is the caller's, because `cn` concatenates
 *  rather than merges and a losing utility would still be in the class list. */
export const fieldClasses = [
  "h-8 rounded-md border border-outline bg-surface-container-high px-2",
  "text-body-md text-on-surface placeholder:text-on-surface-variant",
  "hover:border-on-surface-variant",
  // Focus hugs the field instead of taking the global 2px offset ring, so a
  // field in a tight form does not push a halo over its neighbours.
  "focus-visible:outline-offset-0",
  "disabled:cursor-not-allowed disabled:opacity-45",
  "transition-colors duration-150 ease-out",
].join(" ");

/** A select: the field chrome, with room at the end for the chevron that
 *  `src/styles/controls.css` draws, and one line as tall as the field's inside
 *  (32px less the border) so that stylesheet can clamp a long name to it. */
export const selectClasses = `${fieldClasses} pr-7 leading-[1.875rem]`;

/** A 28px select for a table row or a line of secondary text. */
export const compactSelectClasses = [
  "h-7 rounded-md border border-outline bg-surface-container-high pr-6 pl-1.5",
  "text-body-sm leading-[1.625rem] text-on-surface",
  "hover:border-on-surface-variant",
  "focus-visible:outline-offset-0",
  "disabled:cursor-not-allowed disabled:opacity-45",
  "transition-colors duration-150 ease-out",
].join(" ");

export const fieldLabelClasses = "block text-label-md text-on-surface-variant";
