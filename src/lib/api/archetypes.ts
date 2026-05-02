import { invoke } from "@tauri-apps/api/core";
import type { Archetype, MetricWeight } from "$lib/types/archetype";

/** Create a new archetype. */
export async function createArchetype(
	name: string,
	role: string,
	metrics: MetricWeight[]
): Promise<Archetype> {
	return invoke("create_archetype_cmd", { name, role, metrics });
}

/** List all archetypes for a given role. */
export async function listArchetypesByRole(role: string): Promise<Archetype[]> {
	return invoke("list_archetypes_by_role", { role });
}

/** List all archetypes regardless of role. */
export async function listAllArchetypes(): Promise<Archetype[]> {
	return invoke("list_all_archetypes_cmd");
}

/** Get a single archetype by id. */
export async function getArchetype(id: number): Promise<Archetype> {
	return invoke("get_archetype_cmd", { id });
}

/** Update an existing archetype's name and metrics. */
export async function updateArchetype(
	id: number,
	name: string,
	metrics: MetricWeight[]
): Promise<Archetype> {
	return invoke("update_archetype_cmd", { id, name, metrics });
}

/** Delete an archetype by id. */
export async function deleteArchetype(id: number): Promise<void> {
	return invoke("delete_archetype_cmd", { id });
}
