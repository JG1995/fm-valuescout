import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { moneyballMetric } from "@/utils/moneyball-metrics";
import { MoneyballMetricValue } from "./moneyball-metric-value";

describe("MoneyballMetricValue", () => {
  it("keeps the raw value visible and exposes the coloured percentile tier by name", () => {
    render(
      <dl>
        <MoneyballMetricValue
          metric={moneyballMetric("goals_per_90")}
          value={1.25}
          score={75}
        />
      </dl>,
    );

    expect(screen.getByText("Goals / 90")).toBeInTheDocument();
    expect(screen.getByText("1.25")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "Goals / 90: 75, Good" }),
    ).toBeInTheDocument();
  });

  it("does not turn a missing raw metric into zero or show a score without it", () => {
    render(
      <dl>
        <MoneyballMetricValue
          metric={moneyballMetric("goals")}
          value={null}
          score={null}
        />
      </dl>,
    );

    expect(screen.getByText("—")).toBeInTheDocument();
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });
});
