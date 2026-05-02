import { describe, it, expect, vi, beforeEach } from "vitest";
import "@testing-library/svelte/vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import ArchetypeEditor from "./ArchetypeEditor.svelte";
import { createMockArchetype } from "./archetype.test-utils";

describe("ArchetypeEditor", () => {
	describe("mode header text", () => {
		it("shows Create Archetype when no archetype is provided", () => {
			const { container } = render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});
			expect(container.textContent).toContain("Create Archetype (ST)");
		});

		it("shows Edit Archetype when an archetype is provided", () => {
			const archetype = createMockArchetype({ id: 1, name: "Goal Poacher", role: "ST" });
			const { container } = render(ArchetypeEditor, {
				role: "ST",
				archetype,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});
			expect(container.textContent).toContain("Edit Archetype (ST)");
		});
	});

	describe("name input validation", () => {
		it("shows error when name is empty on save", async () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			const saveBtn = screen.getByTestId("save-btn");
			await fireEvent.click(saveBtn);

			const error = screen.getByTestId("error-message");
			expect(error.textContent).toBe("Name cannot be empty.");
		});
	});

	describe("metric validation", () => {
		it("shows error when a metric key is empty", async () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			const nameInput = screen.getByPlaceholderText("Archetype name");
			await fireEvent.input(nameInput, { target: { value: "Test Archetype" } });

			const keyInput = screen.getByTestId("metric-key-input");
			await fireEvent.input(keyInput, { target: { value: "" } });

			const saveBtn = screen.getByTestId("save-btn");
			await fireEvent.click(saveBtn);

			const error = screen.getByTestId("error-message");
			expect(error.textContent).toBe("All metrics must have a non-empty key.");
		});

		it("shows error when a weight is negative or zero", async () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			const nameInput = screen.getByPlaceholderText("Archetype name");
			await fireEvent.input(nameInput, { target: { value: "Test Archetype" } });

			const keyInput = screen.getByTestId("metric-key-input");
			await fireEvent.input(keyInput, { target: { value: "attacking.goals_per_90" } });

			const weightInput = screen.getByTestId("metric-weight-input");
			await fireEvent.input(weightInput, { target: { value: "-0.5" } });

			const saveBtn = screen.getByTestId("save-btn");
			await fireEvent.click(saveBtn);

			const error = screen.getByTestId("error-message");
			expect(error.textContent).toBe("All weights must be positive.");
		});
	});

	describe("add/remove metric functionality", () => {
		it("adds a metric row when add button is clicked", async () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			const rowsBefore = screen.getAllByTestId("metric-row");
			expect(rowsBefore.length).toBe(1);

			const addBtn = screen.getByTestId("add-metric-btn");
			await fireEvent.click(addBtn);

			const rowsAfter = screen.getAllByTestId("metric-row");
			expect(rowsAfter.length).toBe(2);
		});

		it("removes a metric row when remove button is clicked", async () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			const addBtn = screen.getByTestId("add-metric-btn");
			await fireEvent.click(addBtn);

			const rowsBefore = screen.getAllByTestId("metric-row");
			expect(rowsBefore.length).toBe(2);

			const removeBtns = screen.getAllByTestId("remove-metric-btn");
			await fireEvent.click(removeBtns[0]);

			const rowsAfter = screen.getAllByTestId("metric-row");
			expect(rowsAfter.length).toBe(1);
		});

		it("disables remove button when only one metric remains", () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			const removeBtn = screen.getByTestId("remove-metric-btn");
			expect((removeBtn as HTMLButtonElement).disabled).toBe(true);
		});
	});

	describe("weight total calculation", () => {
		it("displays initial total weight", () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			expect(screen.getByText("Total weight: 1.00")).toBeTruthy();
		});

		it("updates total weight when metric weight changes", async () => {
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			const weightInput = screen.getByTestId("metric-weight-input");
			await fireEvent.input(weightInput, { target: { value: "0.5" } });

			expect(screen.getByText("Total weight: 0.50")).toBeTruthy();
		});
	});

	describe("onsave callback", () => {
		it("calls onsave with correct name and normalized metrics", async () => {
			const mockSave = vi.fn();
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: mockSave,
				onclose: vi.fn(),
			});

			const nameInput = screen.getByPlaceholderText("Archetype name");
			await fireEvent.input(nameInput, { target: { value: "Poacher" } });

			const keyInput = screen.getByTestId("metric-key-input");
			await fireEvent.input(keyInput, { target: { value: "attacking.goals_per_90" } });

			const weightInput = screen.getByTestId("metric-weight-input");
			await fireEvent.input(weightInput, { target: { value: "2" } });

			const saveBtn = screen.getByTestId("save-btn");
			await fireEvent.click(saveBtn);

			expect(mockSave).toHaveBeenCalledOnce();
			expect(mockSave).toHaveBeenCalledWith("Poacher", [
				{ metric_key: "attacking.goals_per_90", weight: 1, inverted: false },
			]);
		});
	});

	describe("onclose callback", () => {
		it("calls onclose when cancel button is clicked", async () => {
			const mockClose = vi.fn();
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: mockClose,
			});

			const cancelBtn = screen.getByTestId("cancel-btn");
			await fireEvent.click(cancelBtn);

			expect(mockClose).toHaveBeenCalledOnce();
		});
	});

	describe("edit mode", () => {
		it("loads existing archetype data in edit mode", () => {
			const archetype = createMockArchetype({
				id: 42,
				name: "Target Man",
				role: "ST",
				metrics: [
					{ metric_key: "attacking.heading_accuracy", weight: 0.7, inverted: false },
					{ metric_key: "attacking.shots_per_90", weight: 0.3, inverted: true },
				],
			});
			render(ArchetypeEditor, {
				role: "ST",
				archetype,
				onsave: vi.fn(),
				onclose: vi.fn(),
			});

			// Name should be pre-filled
			const nameInput = screen.getByPlaceholderText("Archetype name") as HTMLInputElement;
			expect(nameInput.value).toBe("Target Man");

			// Should have two metric rows matching archetype data
			const rows = screen.getAllByTestId("metric-row");
			expect(rows.length).toBe(2);

			// Verify first metric row (key input value)
			const keyInputs = screen.getAllByTestId("metric-key-input");
			expect((keyInputs[0] as HTMLInputElement).value).toBe("attacking.heading_accuracy");
			expect((keyInputs[1] as HTMLInputElement).value).toBe("attacking.shots_per_90");

			// Verify weight inputs (first row weight should be 0.7)
			const weightInputs = screen.getAllByTestId("metric-weight-input");
			expect((weightInputs[0] as HTMLInputElement).value).toBe("0.7");
			expect((weightInputs[1] as HTMLInputElement).value).toBe("0.3");
		});
	});

	describe("backdrop and keyboard interactions", () => {
		it("calls onclose when backdrop is clicked", async () => {
			const mockClose = vi.fn();
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: mockClose,
			});

			// Click on the overlay (backdrop)
			const overlay = screen.getByRole("dialog");
			await fireEvent.click(overlay);

			expect(mockClose).toHaveBeenCalledOnce();
		});

		it("calls onclose when Escape is pressed", async () => {
			const mockClose = vi.fn();
			render(ArchetypeEditor, {
				role: "ST",
				archetype: null,
				onsave: vi.fn(),
				onclose: mockClose,
			});

			// Focus the dialog and press Escape
			const overlay = screen.getByRole("dialog");
			overlay.focus();
			await fireEvent.keyDown(overlay, { key: "Escape" });

			expect(mockClose).toHaveBeenCalledOnce();
		});
	});
});
