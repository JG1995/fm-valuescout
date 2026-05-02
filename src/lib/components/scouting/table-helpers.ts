import type { PlayerScore } from "$lib/scoring/types";

export function getSortValue(score: PlayerScore, key: string): number {
    if (key.startsWith("metric.")) {
        const metricKey = key.slice(7);
        return score.metricPercentiles[metricKey] ?? 0;
    }
    switch (key) {
        case "name":
            return score.name.charCodeAt(0);
        case "rawScore":
            return score.rawScore;
        case "valueAdjustedScore":
            return score.valueAdjustedScore;
        case "age":
            return score.age ?? 0;
        case "transferValue":
            return score.transferValue ?? 0;
        default:
            return 0;
    }
}

export function formatMetricLabel(key: string): string {
    const parts = key.split(".");
    const field = parts[parts.length - 1];
    return field
        .split("_")
        .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
        .join(" ");
}

export function formatValue(value: number | null): string {
    if (value === null) return "—";
    if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
    if (value >= 1_000) return `${Math.floor(value / 1_000)}K`;
    return value.toFixed(0);
}
