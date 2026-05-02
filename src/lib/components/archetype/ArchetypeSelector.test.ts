import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from "vitest";
import "@testing-library/svelte/vitest";
import { render, screen, fireEvent } from "@testing-library/svelte";
import type { Archetype, ArchetypeRole } from "$lib/types/archetype";
import { createMockArchetypes, createMockArchetype } from "./archetype.test-utils";
import ArchetypeSelector from "./ArchetypeSelector.svelte";

describe("ArchetypeSelector", () => {
	let archetypes: Archetype[];
	let onselect: Mock;
	let onedit: Mock;
	let oncreate: Mock;
	let ondelete: Mock;
	let onclose: Mock;

	beforeEach(() => {
		archetypes = createMockArchetypes();
		onselect = vi.fn();
		onedit = vi.fn();
		oncreate = vi.fn();
		ondelete = vi.fn();
		onclose = vi.fn();
		vi.stubGlobal("confirm", vi.fn(() => true));
	});

	afterEach(() => {
		vi.unstubAllGlobals();
	});

	function renderComponent(props: {
		role?: ArchetypeRole;
		archetypes?: Archetype[];
		selectedArchetypeId?: number | null;
	} = {}) {
		return render(ArchetypeSelector, {
			role: props.role ?? "ST",
			archetypes: props.archetypes ?? archetypes,
			selectedArchetypeId: props.selectedArchetypeId ?? null,
			onselect,
			onedit,
			oncreate,
			ondelete,
			onclose,
		});
	}

	describe("rendering", () => {
		it("renders heading with role", () => {
			renderComponent({ role: "GK" });
			expect(screen.getByText("Select Archetype (GK)")).toBeTruthy();
		});

		it("filters archetypes by role", () => {
			renderComponent({ role: "ST" });
			expect(screen.getByText("Goal Poacher")).toBeTruthy();
			expect(screen.getByText("Target Man")).toBeTruthy();
			expect(screen.queryByText("Deep Playmaker")).toBeFalsy();
			expect(screen.queryByText("Sweeper Keeper")).toBeFalsy();
		});

		it("shows only archetypes matching the role", () => {
			renderComponent({ role: "GK" });
			expect(screen.getByText("Sweeper Keeper")).toBeTruthy();
			expect(screen.queryByText("Goal Poacher")).toBeFalsy();
		});
	});

	describe("empty state", () => {
		it("shows empty state when no archetypes match role", () => {
			renderComponent({ role: "AM", archetypes });
			expect(screen.getByText("No archetypes available for this role.")).toBeTruthy();
		});

		it("shows empty state when archetypes array is empty", () => {
			renderComponent({ role: "ST", archetypes: [] });
			expect(screen.getByText("No archetypes available for this role.")).toBeTruthy();
		});
	});

	describe("selection", () => {
		it("calls onselect with archetype when clicked", async () => {
			renderComponent({ role: "ST" });
			const button = screen.getByText("Goal Poacher").closest("button")!;
			await fireEvent.click(button);
			expect(onselect).toHaveBeenCalledTimes(1);
			expect(onselect).toHaveBeenCalledWith(
				expect.objectContaining({ id: 1, name: "Goal Poacher" })
			);
		});

		it("highlights selected archetype", () => {
			const { container } = renderComponent({ role: "ST", selectedArchetypeId: 1 });
			const selectedButton = container.querySelector(".archetype-option.selected");
			expect(selectedButton).toBeTruthy();
			expect(selectedButton?.textContent).toContain("Goal Poacher");
		});

		it("does not highlight unselected archetypes", () => {
			const { container } = renderComponent({ role: "ST", selectedArchetypeId: 2 });
			const selectedButtons = container.querySelectorAll(".archetype-option.selected");
			expect(selectedButtons.length).toBe(1);
			expect(selectedButtons[0]?.textContent).toContain("Target Man");
		});
	});

	describe("edit action", () => {
		it("calls onedit when edit button is clicked", async () => {
			renderComponent({ role: "ST" });
			const editButtons = screen.getAllByLabelText(/Edit/);
			await fireEvent.click(editButtons[0]);
			expect(onedit).toHaveBeenCalledTimes(1);
			expect(onedit).toHaveBeenCalledWith(
				expect.objectContaining({ id: 1, name: "Goal Poacher" })
			);
		});
	});

	describe("delete action", () => {
		it("calls ondelete when delete button is clicked and confirmed", async () => {
			const customArchetype = createMockArchetype({
				id: 10,
				name: "Custom ST",
				role: "ST",
				is_default: false,
			});
			renderComponent({ role: "ST", archetypes: [...archetypes, customArchetype] });
			const deleteButton = screen.getByLabelText("Delete Custom ST");
			await fireEvent.click(deleteButton);
			expect(window.confirm).toHaveBeenCalledWith(
				'Delete archetype "Custom ST"? This cannot be undone.'
			);
			expect(ondelete).toHaveBeenCalledTimes(1);
			expect(ondelete).toHaveBeenCalledWith(
				expect.objectContaining({ id: 10, name: "Custom ST" })
			);
		});

		it("does not call ondelete when confirmation is cancelled", async () => {
			(window.confirm as Mock).mockReturnValue(false);
			const customArchetype = createMockArchetype({
				id: 10,
				name: "Custom ST",
				role: "ST",
				is_default: false,
			});
			renderComponent({ role: "ST", archetypes: [...archetypes, customArchetype] });
			const deleteButton = screen.getByLabelText("Delete Custom ST");
			await fireEvent.click(deleteButton);
			expect(ondelete).not.toHaveBeenCalled();
		});

		it("does not show delete button for default archetypes", () => {
			const defaultArch = createMockArchetype({
				id: 99,
				name: "Default ST",
				role: "ST",
				is_default: true,
			});
			renderComponent({ role: "ST", archetypes: [defaultArch] });
			expect(screen.queryByLabelText("Delete Default ST")).toBeFalsy();
			expect(screen.getByLabelText("Edit Default ST")).toBeTruthy();
		});

		it("shows delete button for non-default archetypes", () => {
			renderComponent({ role: "ST" });
			expect(screen.getByLabelText("Delete Goal Poacher")).toBeTruthy();
			expect(screen.getByLabelText("Delete Target Man")).toBeTruthy();
		});
	});

	describe("create action", () => {
		it("calls oncreate when create button is clicked", async () => {
			renderComponent({ role: "ST" });
			const createButton = screen.getByText("+ Create New Archetype");
			await fireEvent.click(createButton);
			expect(oncreate).toHaveBeenCalledTimes(1);
		});
	});

	describe("close action", () => {
		it("calls onclose when close button is clicked", async () => {
			renderComponent({ role: "ST" });
			const closeButton = screen.getByLabelText("Close");
			await fireEvent.click(closeButton);
			expect(onclose).toHaveBeenCalledTimes(1);
		});

		it("calls onclose when backdrop is clicked", async () => {
			const { container } = renderComponent({ role: "ST" });
			const overlay = container.querySelector(".selector-overlay")!;
			await fireEvent.click(overlay);
			expect(onclose).toHaveBeenCalledTimes(1);
		});

		it("does not call onclose when panel is clicked", async () => {
			const { container } = renderComponent({ role: "ST" });
			const panel = container.querySelector(".selector-panel")!;
			await fireEvent.click(panel);
			expect(onclose).not.toHaveBeenCalled();
		});

		it("calls onclose when Escape is pressed", async () => {
			const { container } = renderComponent({ role: "ST" });
			const overlay = container.querySelector(".selector-overlay")!;
			await fireEvent.keyDown(overlay, { key: "Escape" });
			expect(onclose).toHaveBeenCalledTimes(1);
		});
	});

	describe("default badge", () => {
		it("shows Default badge for default archetypes", () => {
			const defaultArch = createMockArchetype({
				id: 99,
				name: "Default ST",
				role: "ST",
				is_default: true,
			});
			renderComponent({ role: "ST", archetypes: [defaultArch] });
			expect(screen.getByText("Default")).toBeTruthy();
		});

		it("does not show Default badge for custom archetypes", () => {
			renderComponent({ role: "ST" });
			expect(screen.queryByText("Default")).toBeFalsy();
		});
	});
});
