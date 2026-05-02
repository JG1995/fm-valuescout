import { invoke } from "@tauri-apps/api/core";

export interface PlayerSeasonData {
	id: number;
	player_id: number;
	season_id: number;
	fm_uid: number;
	player_name: string;
	club: string | null;
	age: number | null;
	nationality: string | null;
	position: string | null;
	minutes: number | null;
	appearances_started: number | null;
	appearances_sub: number | null;
	wage_per_week: number | null;
	transfer_value_high: number | null;
	data: object | null;
	contract_expires: string | null;
}

export interface LatestSeason {
	id: number;
	save_id: number;
	label: string;
	in_game_date: string;
	player_count: number;
}

/** Get the latest season for the current save. */
export async function getLatestSeason(): Promise<LatestSeason | null> {
	return invoke("get_latest_season");
}

/** Get all players for a season. */
export async function getPlayersForSeason(seasonId: number): Promise<PlayerSeasonData[]> {
	return invoke("get_players_for_season", { seasonId });
}

/** Get a player's career data. */
export async function getPlayerCareer(saveId: number, playerId: number): Promise<PlayerSeasonData[]> {
	return invoke("get_player_career", { saveId, playerId });
}