/**
 * Extract a metric value from a ParsedPlayer data object using a dot-separated key.
 * Key format: "category.field_name" (e.g., "attacking.goals_per_90").
 * Returns null if the path doesn't exist or the value is null/undefined.
 */
export function getMetricValue(
	data: Record<string, unknown> | null,
	key: string,
): number | null {
	if (!data) return null;

	const parts = key.split(".");
	if (parts.length !== 2) return null;

	const [category, field] = parts;
	const categoryObj = data[category];
	if (!categoryObj || typeof categoryObj !== "object") return null;

	const value = (categoryObj as Record<string, unknown>)[field];
	if (value === null || value === undefined) return null;
	if (typeof value !== "number") return null;

	return value;
}
