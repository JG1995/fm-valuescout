import { X } from "lucide-react";
import type { ReactNode } from "react";
import { useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Button } from "@/components/ui/button/button";
import { cn } from "@/utils/cn";

type ModalVariant = "informational" | "form" | "destructive";

type ModalProps = {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
  variant?: ModalVariant;
  className?: string;
  returnFocusTo?: HTMLElement | null;
  fallbackFocusTo?: () => HTMLElement | null;
};

const FOCUSABLE =
  'a[href], button:not([disabled]), textarea, input, select, [tabindex]:not([tabindex="-1"])';
const EXIT_MS = 150;

function getFocusableElements(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
    (element) => !element.hasAttribute("disabled") && element.tabIndex !== -1,
  );
}

export function Modal({
  open,
  onClose,
  title,
  children,
  footer,
  variant = "form",
  className,
  returnFocusTo,
  fallbackFocusTo,
}: ModalProps) {
  const titleId = useId();
  const dialogRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);
  const fallbackFocusToRef = useRef(fallbackFocusTo);
  const shouldReturnFocusRef = useRef(false);
  const [mounted, setMounted] = useState(open);
  const [entered, setEntered] = useState(false);

  onCloseRef.current = onClose;
  fallbackFocusToRef.current = fallbackFocusTo;

  useEffect(() => {
    if (open) {
      setMounted(true);
      const frame = requestAnimationFrame(() => {
        setEntered(true);
      });
      return () => {
        cancelAnimationFrame(frame);
      };
    }

    setEntered(false);
    const timer = window.setTimeout(() => {
      setMounted(false);
    }, EXIT_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [open]);

  useEffect(() => {
    if (!open || !mounted) {
      return;
    }

    triggerRef.current =
      returnFocusTo ?? (document.activeElement as HTMLElement | null);
    shouldReturnFocusRef.current = true;
    const dialog = dialogRef.current;
    const focusables = dialog ? getFocusableElements(dialog) : [];
    focusables[0]?.focus();
  }, [open, mounted, returnFocusTo]);

  useEffect(() => {
    if (!mounted) {
      return;
    }

    const dialog = dialogRef.current;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onCloseRef.current();
        return;
      }

      if (event.key !== "Tab" || !dialog) {
        return;
      }

      const items = getFocusableElements(dialog);
      if (items.length === 0) {
        event.preventDefault();
        return;
      }

      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement;

      if (event.shiftKey && active === first) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && active === last) {
        event.preventDefault();
        first?.focus();
      }
    };

    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [mounted]);

  useEffect(() => {
    if (mounted || !shouldReturnFocusRef.current) {
      return;
    }
    shouldReturnFocusRef.current = false;
    if (triggerRef.current?.isConnected) {
      triggerRef.current.focus();
      return;
    }
    fallbackFocusToRef.current?.()?.focus();
  }, [mounted]);

  if (!mounted) {
    return null;
  }

  const handleBackdropClick = () => {
    if (variant !== "destructive") {
      onCloseRef.current();
    }
  };

  return createPortal(
    <div className="fixed inset-0 z-40 flex items-center justify-center p-4">
      <button
        type="button"
        aria-label="Close dialog"
        className={cn(
          "absolute inset-0 bg-black/60 ease-out motion-reduce:transition-none",
          entered
            ? "opacity-100 transition-opacity duration-200"
            : "opacity-0 transition-opacity duration-150",
        )}
        onClick={handleBackdropClick}
      />
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className={cn(
          "relative z-50 flex max-h-[min(90vh,720px)] w-full max-w-[560px] flex-col",
          "rounded-xl border border-outline-variant bg-surface-container-highest p-6 shadow-overlay",
          "ease-out motion-reduce:translate-y-0 motion-reduce:transition-none",
          entered
            ? "translate-y-0 opacity-100 transition-[opacity,transform] duration-200"
            : "translate-y-2 opacity-0 transition-[opacity,transform] duration-150",
          className,
        )}
      >
        <div className="flex items-start justify-between gap-4">
          <h2 id={titleId} className="text-headline-md text-on-surface">
            {title}
          </h2>
          {variant === "informational" ? (
            <Button
              size="icon"
              variant="ghost"
              icon={X}
              aria-label="Close"
              onClick={() => {
                onCloseRef.current();
              }}
            />
          ) : null}
        </div>
        <div className="mt-4 min-h-0 flex-1 overflow-y-auto">{children}</div>
        {footer ? (
          <div className="mt-6 flex justify-end gap-2">{footer}</div>
        ) : null}
      </div>
    </div>,
    document.body,
  );
}
