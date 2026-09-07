import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  ConfigurableColumnsControl,
  type ConfigurableTableColumn,
  ConfigurableTableHeader,
  type ConfigurableTableIdentityHeader,
  type ConfigurableTableMetric,
} from "./player-table-header";
import type { TableGroupInput } from "./table-groups";

type ConfigurableTableHeaderGroups = TableGroupInput;

const METRICS: ConfigurableTableMetric[] = [
  {
    id: "ca",
    label: "CA",
    align: "right",
    defaultWidth: 72,
    sortable: true,
  },
  {
    id: "suggested_training",
    label: "Suggested Training",
    align: "left",
    defaultWidth: 176,
    sortable: false,
  },
];

const COLUMNS: ConfigurableTableColumn[] = [
  { id: "ca", label: "CA", align: "right", width: 72 },
  {
    id: "suggested_training",
    label: "Suggested Training",
    align: "left",
    width: 176,
  },
];

function renderHeader({
  columns = COLUMNS,
  sortBy = "ca",
  groups,
  fixedColumns,
}: {
  columns?: ConfigurableTableColumn[];
  sortBy?: string;
  groups?: ConfigurableTableHeaderGroups;
  fixedColumns?: ConfigurableTableColumn[];
} = {}) {
  const onSortChange = vi.fn();
  const onAddColumn = vi.fn();
  const onMoveColumn = vi.fn();
  const onRemoveColumn = vi.fn();
  const onResizeColumn = vi.fn();
  render(
    <ConfigurableTableHeader
      columns={columns}
      fixedColumns={fixedColumns}
      groups={groups}
      metrics={METRICS}
      sortBy={sortBy}
      sortDir="desc"
      onSortChange={onSortChange}
      onAddColumn={onAddColumn}
      onRemoveColumn={onRemoveColumn}
      onMoveColumn={onMoveColumn}
      onResizeColumn={onResizeColumn}
    />,
  );
  return {
    onSortChange,
    onAddColumn,
    onMoveColumn,
    onRemoveColumn,
    onResizeColumn,
  };
}

describe("player table header per-column sortability", () => {
  it("gives a non-sortable column no sort click, title, or aria-sort", async () => {
    const user = userEvent.setup();
    const { onSortChange } = renderHeader();

    const header = screen.getByRole("columnheader", {
      name: "Suggested Training",
    });
    expect(header).not.toHaveAttribute("aria-sort");
    const button = within(header).getByRole("button", {
      name: "Suggested Training",
    });
    expect(button).toHaveAttribute("title", "Suggested Training");
    await user.click(button);
    expect(onSortChange).not.toHaveBeenCalled();
  });

  it("keeps sortable columns clickable with title and aria-sort", async () => {
    const user = userEvent.setup();
    const { onSortChange } = renderHeader();

    const header = screen.getByRole("columnheader", { name: "CA" });
    expect(header).toHaveAttribute("aria-sort", "descending");
    const button = within(header).getByRole("button", { name: "CA" });
    expect(button.getAttribute("title")).toContain("click to sort");
    await user.click(button);
    expect(onSortChange).toHaveBeenCalledWith("ca");
  });

  it("lists a valid non-sortable metric in the Add column menu", async () => {
    const user = userEvent.setup();
    const { onAddColumn } = renderHeader({ columns: [COLUMNS[0]] });

    fireEvent.contextMenu(screen.getByRole("columnheader", { name: "CA" }));
    await user.click(screen.getByRole("menuitem", { name: "Add column" }));
    await user.click(
      screen.getByRole("button", { name: "Column: Choose a metric" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search columns" }),
      "training",
    );
    await user.click(
      screen.getByRole("option", { name: "Suggested Training" }),
    );

    expect(onAddColumn).toHaveBeenCalledWith("suggested_training");
  });
});

const GROUPED_COLUMNS: ConfigurableTableColumn[] = [
  { id: "name", label: "Name", align: "left", width: 224 },
  { id: "age", label: "Age / DOB", align: "left", width: 144 },
  { id: "ca", label: "CA", align: "right", width: 72 },
  { id: "value", label: "Value", align: "right", width: 112 },
];

const GROUPED_GROUPS: TableGroupInput = {
  groups: [
    { id: "profile", label: "Profile" },
    { id: "ability", label: "Ability" },
    { id: "market", label: "Market" },
  ],
  groupForColumn: (columnId) => {
    if (columnId === "name" || columnId === "age") return "profile";
    if (columnId === "ca") return "ability";
    if (columnId === "value") return "market";
    return null;
  },
};

function renderIdentityHeader({
  width,
  label = "Player",
}: {
  width?: number;
  label?: string;
} = {}) {
  const onResize = vi.fn();
  const identity: ConfigurableTableIdentityHeader = {
    id: "identity",
    label,
    width,
    onResize,
  };
  render(
    <ConfigurableTableHeader
      columns={COLUMNS}
      metrics={METRICS}
      sortBy="ca"
      sortDir="desc"
      onSortChange={vi.fn()}
      onAddColumn={vi.fn()}
      onRemoveColumn={vi.fn()}
      onMoveColumn={vi.fn()}
      onResizeColumn={vi.fn()}
      identity={identity}
    />,
  );
  return { onResize };
}

describe("player table grouped headers", () => {
  it("renders a group row with colgroup scopes and run spans above the leaf row", () => {
    renderHeader({ columns: GROUPED_COLUMNS, groups: GROUPED_GROUPS });

    const profile = screen.getByRole("columnheader", { name: "Profile" });
    expect(profile).toHaveAttribute("scope", "colgroup");
    expect(profile.getAttribute("colspan")).toBe("2");
    const ability = screen.getByRole("columnheader", { name: "Ability" });
    expect(ability.getAttribute("colspan")).toBe("1");
    const market = screen.getByRole("columnheader", { name: "Market" });
    expect(market.getAttribute("colspan")).toBe("1");

    const thead = profile.closest("thead");
    expect(thead?.querySelectorAll("tr")).toHaveLength(2);
    const leaf = screen.getByRole("columnheader", { name: "CA" });
    expect(leaf).toHaveAttribute("aria-sort", "descending");
  });

  it("centers group labels over their span", () => {
    renderHeader({ columns: GROUPED_COLUMNS, groups: GROUPED_GROUPS });

    for (const name of ["Profile", "Ability", "Market"]) {
      const group = screen.getByRole("columnheader", { name });
      expect(group).toHaveAttribute("scope", "colgroup");
      expect(group.className).toContain("text-center");
    }
  });

  it("repeats an interrupted group as a new run after a move", () => {
    renderHeader({
      columns: [GROUPED_COLUMNS[0], GROUPED_COLUMNS[2], GROUPED_COLUMNS[1]],
      groups: GROUPED_GROUPS,
    });

    const profiles = screen.getAllByRole("columnheader", { name: "Profile" });
    expect(profiles).toHaveLength(2);
    expect(profiles[0].getAttribute("colspan")).toBe("1");
    expect(profiles[1].getAttribute("colspan")).toBe("1");
    expect(
      screen.getByRole("columnheader", { name: "Ability" }),
    ).toBeInTheDocument();
  });

  it("renders no group cell for a group with no visible leaves", () => {
    renderHeader({
      columns: [GROUPED_COLUMNS[0], GROUPED_COLUMNS[1]],
      groups: GROUPED_GROUPS,
    });

    expect(
      screen.getByRole("columnheader", { name: "Profile" }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: "Ability" })).toBeNull();
    expect(screen.queryByRole("columnheader", { name: "Market" })).toBeNull();
  });

  it("assigns unmapped leaves to the Other fallback group", () => {
    renderHeader({
      columns: [
        GROUPED_COLUMNS[0],
        { id: "mystery", label: "Mystery", align: "left", width: 120 },
      ],
      groups: GROUPED_GROUPS,
    });

    const other = screen.getByRole("columnheader", { name: "Other" });
    expect(other).toHaveAttribute("scope", "colgroup");
    expect(other.getAttribute("colspan")).toBe("1");
    expect(
      screen.getByRole("columnheader", { name: "Profile" }),
    ).toBeInTheDocument();
  });

  it("keeps transitional identity leaves in Profile with sort, menu, and resize", async () => {
    const user = userEvent.setup();
    const { onSortChange, onResizeColumn } = renderHeader({
      columns: [
        { id: "name", label: "Name", align: "left", width: 224 },
        { id: "club", label: "Club", align: "left", width: 192 },
        { id: "division", label: "Division", align: "left", width: 168 },
      ],
      sortBy: "name",
      groups: {
        groups: [{ id: "profile", label: "Profile" }],
        groupForColumn: (columnId) =>
          ["name", "club", "division"].includes(columnId) ? "profile" : null,
      },
    });

    const profile = screen.getByRole("columnheader", { name: "Profile" });
    expect(profile.getAttribute("colspan")).toBe("3");
    expect(screen.queryByRole("columnheader", { name: "Other" })).toBeNull();

    const nameHeader = screen.getByRole("columnheader", { name: "Name" });
    expect(nameHeader).toHaveAttribute("aria-sort", "descending");
    await user.click(within(nameHeader).getByRole("button", { name: "Name" }));
    expect(onSortChange).toHaveBeenCalledWith("name");

    fireEvent.contextMenu(screen.getByRole("columnheader", { name: "Club" }));
    expect(
      screen.getByRole("menuitem", { name: "Move left" }),
    ).toBeInTheDocument();

    expect(
      screen.getByRole("separator", { name: "Resize Division column" }),
    ).toBeInTheDocument();
    expect(onResizeColumn).not.toHaveBeenCalled();
  });

  it("gives fixed action columns rowSpan 2 above the leaf row", () => {
    renderHeader({
      columns: [GROUPED_COLUMNS[0]],
      groups: GROUPED_GROUPS,
      fixedColumns: [
        { id: "actions", label: "Actions", align: "left", width: 128 },
      ],
    });

    const actions = screen.getByRole("columnheader", { name: "Actions" });
    expect(actions.getAttribute("rowspan")).toBe("2");
    expect(actions.closest("thead")?.querySelectorAll("tr")).toHaveLength(2);
    fireEvent.contextMenu(actions);
    expect(screen.queryByRole("menu")).toBeNull();
  });

  it("renders the sole identity header without sort, menu, or remove affordances", () => {
    const onResize = vi.fn();
    render(
      <ConfigurableTableHeader
        columns={GROUPED_COLUMNS}
        groups={GROUPED_GROUPS}
        metrics={METRICS}
        sortBy="ca"
        sortDir="desc"
        onSortChange={vi.fn()}
        onAddColumn={vi.fn()}
        onRemoveColumn={vi.fn()}
        onMoveColumn={vi.fn()}
        onResizeColumn={vi.fn()}
        identity={{
          id: "identity",
          label: "Player",
          width: 280,
          onResize,
        }}
      />,
    );

    const identity = screen.getByRole("columnheader", { name: "Player" });
    expect(identity).toHaveAttribute("scope", "col");
    expect(identity.getAttribute("rowspan")).toBe("2");
    expect(within(identity).queryByRole("button")).toBeNull();
    fireEvent.contextMenu(identity);
    expect(screen.queryByRole("menu")).toBeNull();
    expect(screen.queryByRole("menuitem")).toBeNull();
    // The identity corner leads the group row ahead of every group run.
    const thead = identity.closest("thead") as HTMLElement;
    expect(thead.querySelector("tr")?.firstElementChild).toBe(identity);
    expect(onResize).not.toHaveBeenCalled();
  });

  it("exposes the 240/280/360 identity resize bounds", () => {
    renderIdentityHeader({ width: 280 });

    const handle = screen.getByRole("separator", {
      name: "Resize Player column",
    });
    expect(handle).toHaveAttribute("aria-valuemin", "240");
    expect(handle).toHaveAttribute("aria-valuemax", "360");
    expect(handle).toHaveAttribute("aria-valuenow", "280");
  });

  it("clamps identity keyboard resize at both bounds", () => {
    const { onResize } = renderIdentityHeader({ width: 280 });
    const handle = screen.getByRole("separator", {
      name: "Resize Player column",
    });

    fireEvent.keyDown(handle, { key: "ArrowLeft" });
    expect(onResize).toHaveBeenLastCalledWith(264);
    fireEvent.keyDown(handle, { key: "ArrowRight" });
    expect(onResize).toHaveBeenLastCalledWith(296);
    fireEvent.keyDown(handle, { key: "Home" });
    expect(onResize).toHaveBeenLastCalledWith(240);
    fireEvent.keyDown(handle, { key: "End" });
    expect(onResize).toHaveBeenLastCalledWith(360);
  });

  it("honors the identity bounds when stepping past them", () => {
    const below = renderIdentityHeader({ width: 240 });
    fireEvent.keyDown(
      screen.getByRole("separator", { name: "Resize Player column" }),
      { key: "ArrowLeft" },
    );
    expect(below.onResize).toHaveBeenCalledWith(240);
    cleanup();

    const above = renderIdentityHeader({ width: 360 });
    fireEvent.keyDown(
      screen.getByRole("separator", { name: "Resize Player column" }),
      { key: "ArrowRight" },
    );
    expect(above.onResize).toHaveBeenCalledWith(360);
  });

  it("clamps identity pointer resize at both bounds", () => {
    const { onResize } = renderIdentityHeader({ width: 280 });
    const handle = screen.getByRole("separator", {
      name: "Resize Player column",
    });

    fireEvent.pointerDown(handle, { pointerId: 1, clientX: 200 });
    fireEvent.pointerMove(handle, { pointerId: 1, clientX: -100 });
    expect(onResize).toHaveBeenLastCalledWith(240);
    fireEvent.pointerMove(handle, { pointerId: 1, clientX: 800 });
    expect(onResize).toHaveBeenLastCalledWith(360);
    fireEvent.pointerUp(handle, { pointerId: 1 });
  });

  it("defaults the identity width to 280 without a supplied width", () => {
    renderIdentityHeader({ width: undefined });

    expect(
      screen.getByRole("separator", { name: "Resize Player column" }),
    ).toHaveAttribute("aria-valuenow", "280");
  });

  it("keeps a non-sortable leaf sort-free under its group", async () => {
    const user = userEvent.setup();
    const { onSortChange } = renderHeader({
      columns: [
        { id: "ca", label: "CA", align: "right", width: 72 },
        {
          id: "suggested_training",
          label: "Suggested Training",
          align: "left",
          width: 176,
        },
      ],
      groups: {
        groups: [
          { id: "ability", label: "Ability" },
          { id: "development", label: "Development" },
        ],
        groupForColumn: (columnId) =>
          columnId === "ca" ? "ability" : "development",
      },
    });

    expect(
      screen.getByRole("columnheader", { name: "Development" }),
    ).toBeInTheDocument();
    const header = screen.getByRole("columnheader", {
      name: "Suggested Training",
    });
    expect(header).not.toHaveAttribute("aria-sort");
    await user.click(
      within(header).getByRole("button", { name: "Suggested Training" }),
    );
    expect(onSortChange).not.toHaveBeenCalled();
  });
});

const CONTROL_METRICS: ConfigurableTableMetric[] = [
  {
    id: "age",
    label: "Age / DOB",
    align: "left",
    defaultWidth: 144,
    sortable: true,
  },
  {
    id: "ca",
    label: "CA",
    align: "right",
    defaultWidth: 72,
    sortable: true,
  },
  {
    id: "value",
    label: "Value",
    align: "right",
    defaultWidth: 112,
    sortable: true,
  },
  {
    id: "mystery",
    label: "Mystery",
    align: "left",
    defaultWidth: 120,
    sortable: true,
  },
];

function renderColumnsControl({
  visibleColumnIds = ["ca"],
  metrics = CONTROL_METRICS,
  groups = GROUPED_GROUPS,
  configurable = true,
}: {
  visibleColumnIds?: readonly string[];
  metrics?: ConfigurableTableMetric[];
  groups?: TableGroupInput;
  configurable?: boolean;
} = {}) {
  const onAddColumn = vi.fn();
  const onRemoveColumn = vi.fn();
  render(
    <ConfigurableColumnsControl
      groups={groups}
      metrics={metrics}
      visibleColumnIds={visibleColumnIds}
      configurable={configurable}
      onAddColumn={onAddColumn}
      onRemoveColumn={onRemoveColumn}
    />,
  );
  return { onAddColumn, onRemoveColumn };
}

async function openColumnsControl() {
  const user = userEvent.setup();
  await user.click(screen.getByRole("button", { name: "Columns" }));
  const dialog = screen.getByRole("dialog", { name: "Columns" });
  return { user, dialog };
}

describe("grouped columns control", () => {
  it("lists optional analysis columns grouped per the same view map as the header", async () => {
    renderColumnsControl({ visibleColumnIds: ["ca"] });
    const { dialog } = await openColumnsControl();

    // Display-priority order matches the view input, Other fallback last.
    const sections = Array.from(dialog.querySelectorAll("fieldset")).map(
      (fieldset) => fieldset.querySelector("legend")?.textContent,
    );
    expect(sections).toEqual(["Profile", "Ability", "Market", "Other"]);
    expect(
      within(dialog).getByRole("checkbox", { name: "Age / DOB" }),
    ).not.toBeChecked();
    expect(within(dialog).getByRole("checkbox", { name: "CA" })).toBeChecked();
    expect(
      within(dialog).getByRole("checkbox", { name: "Value" }),
    ).not.toBeChecked();
    expect(
      within(dialog).getByRole("checkbox", { name: "Mystery" }),
    ).not.toBeChecked();
  });

  it("offers no group section for a group with no available leaves", async () => {
    renderColumnsControl({
      metrics: CONTROL_METRICS.filter((metric) => metric.id === "ca"),
    });
    const { dialog } = await openColumnsControl();

    expect(
      within(dialog).queryByRole("checkbox", { name: "Age / DOB" }),
    ).toBeNull();
    expect(within(dialog).getByText("Ability")).toBeInTheDocument();
    expect(within(dialog).queryByText("Profile")).toBeNull();
    expect(within(dialog).queryByText("Market")).toBeNull();
    expect(within(dialog).queryByText("Other")).toBeNull();
  });

  it("toggles optional columns through the keyboard", async () => {
    const { onAddColumn, onRemoveColumn } = renderColumnsControl({
      visibleColumnIds: ["ca"],
    });
    const { user, dialog } = await openColumnsControl();

    within(dialog).getByRole("checkbox", { name: "Value" }).focus();
    await user.keyboard(" ");
    expect(onAddColumn).toHaveBeenCalledWith("value");
    expect(onRemoveColumn).not.toHaveBeenCalled();

    within(dialog).getByRole("checkbox", { name: "CA" }).focus();
    await user.keyboard(" ");
    expect(onRemoveColumn).toHaveBeenCalledWith("ca");
  });

  it("lets the last visible analysis column be removed for identity-only", async () => {
    const { onRemoveColumn } = renderColumnsControl({
      visibleColumnIds: ["ca"],
    });
    const { user, dialog } = await openColumnsControl();

    const lastChecked = within(dialog).getByRole("checkbox", {
      name: "CA",
    });
    expect(lastChecked).toBeChecked();
    expect(lastChecked).toBeEnabled();
    await user.click(lastChecked);
    expect(onRemoveColumn).toHaveBeenCalledWith("ca");
  });

  it("offers no toggles when the layout is fixed", () => {
    renderColumnsControl({ configurable: false });

    expect(screen.queryByRole("button", { name: "Columns" })).toBeNull();
    expect(screen.queryByRole("dialog", { name: "Columns" })).toBeNull();
  });

  it("closes the grouped list with Escape and returns focus", async () => {
    renderColumnsControl({ visibleColumnIds: ["ca"] });
    const { user, dialog } = await openColumnsControl();

    within(dialog).getByRole("checkbox", { name: "CA" }).focus();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Columns" })).toBeNull();
    expect(screen.getByRole("button", { name: "Columns" })).toHaveFocus();
  });

  it("focuses the first checkbox on keyboard open so Escape closes and restores focus", async () => {
    renderColumnsControl({ visibleColumnIds: ["ca"] });
    const user = userEvent.setup();

    await user.tab();
    expect(screen.getByRole("button", { name: "Columns" })).toHaveFocus();
    await user.keyboard("{Enter}");

    const dialog = screen.getByRole("dialog", { name: "Columns" });
    expect(
      within(dialog).getByRole("checkbox", { name: "Age / DOB" }),
    ).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Columns" })).toBeNull();
    expect(screen.getByRole("button", { name: "Columns" })).toHaveFocus();
  });
});

describe("analysis leaf removal", () => {
  it("lets the leaf menu remove the last analysis column for identity-only", async () => {
    const user = userEvent.setup();
    const { onRemoveColumn } = renderHeader({ columns: [COLUMNS[0]] });

    fireEvent.contextMenu(screen.getByRole("columnheader", { name: "CA" }));
    const remove = screen.getByRole("menuitem", { name: "Remove CA" });
    expect(remove).toBeEnabled();
    await user.click(remove);
    expect(onRemoveColumn).toHaveBeenCalledWith("ca");
  });
});

describe("compact tactic leaf headers", () => {
  const FULL = "GK (Goalkeeper) / GK (Line-Holding Keeper)";
  const TACTIC_COLUMNS: ConfigurableTableColumn[] = [
    {
      id: "tactic_current.goalkeeper",
      label: "GK",
      secondaryLabel: "Goalkeeper / Line-Holding Keeper",
      accessibleLabel: FULL,
      align: "right",
      width: 112,
    },
  ];

  it("shows compact primary with role context while the focusable leaf keeps the full definition", async () => {
    const user = userEvent.setup();
    renderHeader({ columns: TACTIC_COLUMNS });

    const header = screen.getByRole("columnheader", { name: FULL });
    expect(within(header).getByText("GK")).toBeVisible();
    expect(
      within(header).getByText("Goalkeeper / Line-Holding Keeper"),
    ).toBeVisible();
    // The focusable leaf itself carries the complete definition.
    within(header).getByRole("button", { name: FULL });
    // No visible definition until keyboard focus — title alone is no proof.
    expect(within(header).queryByRole("tooltip")).toBeNull();

    await user.tab();
    expect(
      await within(header).findByRole("tooltip", { name: FULL }),
    ).toBeVisible();
  });

  it("hides the visible definition again after focus leaves the leaf", async () => {
    const user = userEvent.setup();
    renderHeader({ columns: TACTIC_COLUMNS });

    const header = screen.getByRole("columnheader", { name: FULL });
    await user.tab();
    expect(
      await within(header).findByRole("tooltip", { name: FULL }),
    ).toBeVisible();

    await user.tab();
    expect(within(header).queryByRole("tooltip")).toBeNull();
  });
});

describe("compact nationality leaf header", () => {
  const NATIONALITY_COLUMNS: ConfigurableTableColumn[] = [
    {
      id: "nationality",
      label: "Nat.",
      accessibleLabel: "Nationality",
      align: "left",
      width: 128,
    },
  ];

  it("shows Nat. while keeping Nationality for the accessible name, disclosure, sort, and menu", async () => {
    const user = userEvent.setup();
    renderHeader({ columns: NATIONALITY_COLUMNS, sortBy: "nationality" });

    const header = screen.getByRole("columnheader", {
      name: "Nationality",
    });
    expect(within(header).getByText("Nat.")).toBeVisible();
    const button = within(header).getByRole("button", {
      name: "Nationality",
    });
    expect(button.getAttribute("title")).toContain("Nationality");
    expect(within(header).queryByRole("tooltip")).toBeNull();

    await user.hover(header);
    expect(
      await within(header).findByRole("tooltip", { name: "Nationality" }),
    ).toBeVisible();
    await user.unhover(header);
    expect(within(header).queryByRole("tooltip")).toBeNull();

    await user.tab();
    expect(button).toHaveFocus();
    expect(
      await within(header).findByRole("tooltip", { name: "Nationality" }),
    ).toBeVisible();
    await user.tab();
    expect(within(header).queryByRole("tooltip")).toBeNull();

    fireEvent.contextMenu(header);
    expect(
      screen.getByRole("menuitem", { name: "Remove Nationality" }),
    ).toBeInTheDocument();
  });
});
