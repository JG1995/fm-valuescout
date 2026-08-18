import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  component: IndexPage,
});

function IndexPage() {
  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Dashboard</h1>
      <p className="text-body-md text-on-surface-variant">Placeholder.</p>
    </div>
  );
}
