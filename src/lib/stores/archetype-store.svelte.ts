import type { Archetype, ArchetypeRole, MetricWeight } from "$lib/types/archetype";
import * as api from "$lib/api/archetypes";

/** Module-level state using Svelte 5 runes */
let archetypes = $state<Archetype[]>([]);
let selectedArchetypes = $state<Record<string, Archetype | undefined>>({});
let error = $state<string | null>(null);
let loading = $state(false);

/** Group archetypes by role, derived from archetypes state */
const groupedByRole = $derived.by(() => {
	const grouped: Record<string, Archetype[]> = {};
	for (const arch of archetypes) {
		if (!grouped[arch.role]) {
			grouped[arch.role] = [];
		}
		grouped[arch.role].push(arch);
	}
	return grouped;
});

/** Load all archetypes from the backend */
async function loadAll(): Promise<void> {
	loading = true;
	error = null;
	try {
		archetypes = await api.listAllArchetypes();
	} catch (err) {
		error = err instanceof Error ? err.message : "Failed to load archetypes";
		throw err;
	} finally {
		loading = false;
	}
}

/** Load archetypes filtered by role */
async function loadByRole(role: string): Promise<void> {
	loading = true;
	error = null;
	try {
		archetypes = await api.listArchetypesByRole(role);
	} catch (err) {
		error = err instanceof Error ? err.message : "Failed to load archetypes";
		throw err;
	} finally {
		loading = false;
	}
}

/** Create a new archetype */
async function create(
	name: string,
	role: string,
	metrics: MetricWeight[]
): Promise<Archetype> {
	error = null;
	try {
		const archetype = await api.createArchetype(name, role, metrics);
		archetypes = [...archetypes, archetype];
		return archetype;
	} catch (err) {
		error = err instanceof Error ? err.message : "Failed to create archetype";
		throw err;
	}
}

/** Update an existing archetype */
async function update(
	id: number,
	name: string,
	metrics: MetricWeight[]
): Promise<Archetype> {
	error = null;
	try {
		const archetype = await api.updateArchetype(id, name, metrics);
		archetypes = archetypes.map((a) => (a.id === id ? archetype : a));
		return archetype;
	} catch (err) {
		error = err instanceof Error ? err.message : "Failed to update archetype";
		throw err;
	}
}

/** Delete an archetype */
async function remove(id: number): Promise<void> {
	error = null;
	try {
		await api.deleteArchetype(id);
		archetypes = archetypes.filter((a) => a.id !== id);
		// Remove from selected archetypes too
		const newSelected: Record<string, Archetype | undefined> = {};
		for (const [slot, arch] of Object.entries(selectedArchetypes)) {
			if (arch?.id !== id) {
				newSelected[slot] = arch;
			}
		}
		selectedArchetypes = newSelected;
	} catch (err) {
		error = err instanceof Error ? err.message : "Failed to delete archetype";
		throw err;
	}
}

/** Select an archetype for a specific slot */
async function selectArchetype(archetypeId: number, slot: string): Promise<void> {
	error = null;
	try {
		const archetype = await api.getArchetype(archetypeId);
		selectedArchetypes = { ...selectedArchetypes, [slot]: archetype };
	} catch (err) {
		error =
			err instanceof Error ? err.message : "Failed to select archetype";
		throw err;
	}
}

/** Get archetypes for a specific role */
function getArchetypesForRole(role: string): Archetype[] {
	return groupedByRole[role] ?? [];
}

/** Get the selected archetype for a slot */
function getSelectedForSlot(slot: string): Archetype | undefined {
	return selectedArchetypes[slot];
}

/** Clear error state */
function clearError(): void {
	error = null;
}

/** Reset store to initial state */
function reset(): void {
	archetypes = [];
	selectedArchetypes = {};
	error = null;
	loading = false;
}

/** Export store interface */
export function getArchetypeStore() {
	return {
		get archetypes() {
			return archetypes;
		},
		get archetypesByRole() {
			return groupedByRole;
		},
		get selectedArchetypes() {
			return selectedArchetypes;
		},
		get error() {
			return error;
		},
		get loading() {
			return loading;
		},
		loadAll,
		loadByRole,
		create,
		update,
		remove,
		selectArchetype,
		getArchetypesForRole,
		getSelectedForSlot,
		clearError,
		reset,
	};
}
