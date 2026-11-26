import { Info } from "lucide-react";
import {
  cloneElement,
  type ReactElement,
  type ReactNode,
  useEffect,
  useId,
  useRef,
  useState,
} from "react";
import { cn } from "@/utils/cn";

/** A pointer must rest on the trigger this long; keyboard focus shows at once. */
const HOVER_DELAY_MS = 300;
/** Lets the pointer cross the gap between the trigger and the tooltip. */
const LEAVE_GRACE_MS = 100;
const GAP = 6;
const VIEWPORT_PADDING = 8;

type Describable = { "aria-describedby"?: string; tabIndex?: number };

/** Whether focus came from the keyboard; a browser without the selector counts as yes. */
function focusIsVisible(target: Element) {
  try {
    return target.matches(":focus-visible");
  } catch {
    return true;
  }
}

/** Whether a select in the trigger has its list open, which the tooltip would cover. */
function hasOpenList(wrapper: Element | null) {
  try {
    return wrapper?.querySelector("select:open") != null;
  } catch {
    return false;
  }
}

type TooltipProps = {
  /** What the trigger cannot say itself. Nothing the user must read to act. */
  content: ReactNode;
  /**
   * The element the tooltip explains. It takes `aria-describedby`; pass a
   * function to place that id on a control inside it, such as the checkbox of
   * a label.
   */
  children:
    | ReactElement<Describable>
    | ((tooltipId: string) => ReactElement<Describable>);
  /**
   * Makes a trigger that is not a control reachable by keyboard. Leave it off
   * for buttons, links, and fields, which are focusable already.
   */
  focusable?: boolean;
  /**
   * The content is the trigger's accessible name, as on an icon-only button.
   * It is shown but not announced again, and exists only while it is open.
   */
  repeatsName?: boolean;
  className?: string;
};

/**
 * An explanation that appears while its trigger is hovered or focused, and
 * leaves on Escape, on scroll, and when the pointer or focus moves away.
 */
export function Tooltip({
  content,
  children,
  focusable = false,
  repeatsName = false,
  className,
}: TooltipProps) {
  const tooltipId = useId();
  const wrapperRef = useRef<HTMLSpanElement>(null);
  const tooltipRef = useRef<HTMLSpanElement>(null);
  const timer = useRef<number | undefined>(undefined);
  const [open, setOpen] = useState(false);
  const supportsPopover =
    typeof HTMLElement !== "undefined" &&
    "showPopover" in HTMLElement.prototype;

  const schedule = (next: boolean, delay: number) => {
    window.clearTimeout(timer.current);
    const apply = () => setOpen(next && !hasOpenList(wrapperRef.current));
    if (delay === 0) {
      apply();
      return;
    }
    timer.current = window.setTimeout(apply, delay);
  };

  useEffect(() => () => window.clearTimeout(timer.current), []);

  useEffect(() => {
    if (!open) return;
    const close = () => setOpen(false);
    const onKeyDown = (event: KeyboardEvent) => {
      // A key pressed in a select opens or moves through its list.
      if (event.key === "Escape" || event.target instanceof HTMLSelectElement) {
        close();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    window.addEventListener("scroll", close, true);
    window.addEventListener("resize", close);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("scroll", close, true);
      window.removeEventListener("resize", close);
    };
  }, [open]);

  useEffect(() => {
    const tooltip = tooltipRef.current;
    // The wrapper takes no box of its own, so the trigger is measured.
    const anchor = wrapperRef.current?.firstElementChild;
    if (!open || !tooltip || !anchor || !supportsPopover) return;

    tooltip.showPopover();
    const anchorBox = anchor.getBoundingClientRect();
    const box = tooltip.getBoundingClientRect();
    // Centred under a narrow trigger; a trigger wider than the tooltip, such
    // as a label that fills its row, keeps the tooltip at its start instead.
    const start =
      anchorBox.width > box.width
        ? anchorBox.left
        : anchorBox.left + anchorBox.width / 2 - box.width / 2;
    const left = Math.min(
      Math.max(VIEWPORT_PADDING, start),
      window.innerWidth - box.width - VIEWPORT_PADDING,
    );
    const below = anchorBox.bottom + GAP;
    const above = anchorBox.top - GAP - box.height;
    const fitsBelow =
      below + box.height <= window.innerHeight - VIEWPORT_PADDING;
    tooltip.style.left = `${Math.max(VIEWPORT_PADDING, left)}px`;
    tooltip.style.top = `${fitsBelow || above < VIEWPORT_PADDING ? below : above}px`;
    return () => tooltip.hidePopover();
  }, [open, supportsPopover]);

  const trigger =
    typeof children === "function"
      ? children(tooltipId)
      : repeatsName
        ? children
        : cloneElement(children, { "aria-describedby": tooltipId });

  return (
    // biome-ignore lint/a11y/noStaticElementInteractions: the wrapper only listens for the hover and focus of the control inside it.
    <span
      ref={wrapperRef}
      className="contents"
      onPointerEnter={() => schedule(true, HOVER_DELAY_MS)}
      onPointerLeave={() => schedule(false, LEAVE_GRACE_MS)}
      // A press means the user is acting, and may open a list right here.
      onPointerDown={() => schedule(false, 0)}
      onFocus={(event) => {
        // A click focuses the trigger too; only keyboard focus opens at once.
        if (focusIsVisible(event.target)) schedule(true, 0);
      }}
      onBlur={() => schedule(false, 0)}
    >
      {focusable ? cloneElement(trigger, { tabIndex: 0 }) : trigger}
      {repeatsName && !open ? null : (
        <span
          ref={tooltipRef}
          id={tooltipId}
          role="tooltip"
          aria-hidden={repeatsName || undefined}
          popover={supportsPopover ? "manual" : undefined}
          hidden={supportsPopover ? undefined : !open}
          className={cn(
            "fixed inset-auto z-50 m-0 w-max max-w-72 overflow-visible rounded-md border border-outline-variant bg-surface-container-highest px-2.5 py-1.5",
            "text-left text-body-sm font-normal normal-case tracking-normal whitespace-normal text-on-surface shadow-overlay",
            className,
          )}
        >
          {content}
        </span>
      )}
    </span>
  );
}

/**
 * A quiet "i" that explains something with no control of its own to hang a
 * tooltip on. Prefer `Tooltip` on the control, label, or heading itself.
 */
export function InfoTip({
  label,
  children,
}: {
  /** Names the button, such as "About league-adjusted scores". */
  label: string;
  children: ReactNode;
}) {
  return (
    <Tooltip content={children}>
      <button
        type="button"
        aria-label={label}
        className="inline-flex size-6 shrink-0 items-center justify-center rounded-sm text-on-surface-variant transition-colors duration-150 ease-out hover:text-on-surface"
      >
        <Info aria-hidden size={14} strokeWidth={1.5} />
      </button>
    </Tooltip>
  );
}
