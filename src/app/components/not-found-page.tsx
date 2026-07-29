import { Link } from "@tanstack/react-router";

export function NotFoundPage() {
  return (
    <section className="space-y-2">
      <h1 className="text-headline-lg text-on-surface">Page not found</h1>
      <p className="text-body-md text-on-surface-variant">
        That route does not exist in this app.
      </p>
      <Link
        to="/"
        className="inline-flex h-8 cursor-pointer items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high"
      >
        Back to dashboard
      </Link>
    </section>
  );
}
