import { invoke } from "@tauri-apps/api/core";
import { scoreAllPlayers } from "$lib/scoring";
import type { PlayerScore, ScorablePlayer } from "$lib/scoring/types";
import type { Archetype } from "$lib/types/archetype";

/** Raw player data returned from Rust backend */
export interface PlayerSeasonData {
	id: number;
	player_id: number;
	season_id: number;
	fm_uid: number;
	player_name: string;
	club: string | null;
	age: number | null;
	nationality: string | null;
	position: string;
	minutes: number | null;
	transfer_value_high: number | null;
	data: Record<string, unknown> | null;
}

/** Module-level state using Svelte 5 runes */
let players = $state<ScorablePlayer[]>([]);
let scores = $state<PlayerScore[]>([]);
let activeArchetype = $state<Archetype | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);
let seasonId = $state<number | null>(null);

/** Convert backend PlayerSeasonData to ScorablePlayer */
export function toScorable(psd: PlayerSeasonData): ScorablePlayer {
	return {
		playerId: psd.id,
		fmUid: psd.fm_uid,
		name: psd.player_name,
		club: psd.club,
		positions: psd.position,
		age: psd.age,
		transferValueHigh: psd.transfer_value_high,
		data: psd.data,
	};
}

/** Load players for a given season, optionally re-score with active archetype */
async function loadPlayers(seasonIdParam: number): Promise<void> {
	loading = true;
	error = null;
	try {
		const rawPlayers = await invoke<PlayerSeasonData[]>("get_players_for_season", {
			seasonId: seasonIdParam,
		});
		players = rawPlayers.map(toScorable);
		seasonId = seasonIdParam;

		// Re-score if an archetype is already selected
		if (activeArchetype) {
			scores = scoreAllPlayers(players, activeArchetype);
		}
	} catch (err) {
		error = err instanceof Error ? err.message : "Failed to load players";
		throw err;
	} finally {
		loading = false;
	}
}

/** Select an archetype and score all loaded players */
function selectArchetype(archetype: Archetype | null): void {
	activeArchetype = archetype;
	if (archetype && players.length > 0) {
		scores = scoreAllPlayers(players, archetype);
	} else {
		scores = [];
	}
}

/** Clear error state */
function clearError(): void {
	error = null;
}

/** Clear all module-level state */
function reset(): void {
	players = [];
	scores = [];
	activeArchetype = null;
	loading = false;
	error = null;
	seasonId = null;
}

/** Export store interface */
export function getScoutingStore() {
	return {
		get players() {
			return players;
		},
		get scores() {
			return scores;
		},
		get activeArchetype() {
			return activeArchetype;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		},
		get seasonId() {
			return seasonId;
		},
		loadPlayers,
		selectArchetype,
		clearError,
		reset,
	};
}
