/** Shared control chrome for every form field, so a text input and a select
 *  cannot drift apart visually. */
export const fieldClasses = [
  "h-8 w-full rounded-md border border-outline bg-surface-container-high px-2",
  "text-body-md text-on-surface placeholder:text-on-surface-variant",
  "hover:border-on-surface-variant",
  // Focus hugs the field instead of taking the global 2px offset ring, so a
  // field in a tight form does not push a halo over its neighbours.
  "focus-visible:outline-offset-0",
  "disabled:cursor-not-allowed disabled:opacity-45",
  "transition-colors duration-150 ease-out",
].join(" ");

export const fieldLabelClasses = "block text-label-md text-on-surface-variant";
