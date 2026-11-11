//! The private reference harness behind `./scripts/dev save-stats-reference`.
//!
//! It reads the save named by `FM_VALUESCOUT_SAVE_FILE` and prints one JSON report of the
//! save's facts, each shortlist's name and player count, each shortlisted staff member's
//! Unique ID, Preferred Job, and Coaching Qualifications, and every player's totals, keyed by
//! Unique ID with fmsave's field names, so the totals can be compared with fmsave's outside
//! the repository. The report never holds the path, and a missing or unreadable file fails
//! without printing it.

use std::path::PathBuf;
use std::time::Instant;

use serde_json::{json, Map, Value};

use super::{read_season_stats, SavePlayerSeasonStats, SeasonStat};

fn player_report(player: &SavePlayerSeasonStats) -> Value {
    let mut fields = Map::new();
    fields.insert("starts".into(), player.starts.into());
    fields.insert(
        "substitute_appearances".into(),
        player.substitute_appearances.into(),
    );
    fields.insert("minutes".into(), player.minutes.into());
    fields.insert("rated_appearances".into(), player.rated_appearances.into());
    fields.insert("rating_sum".into(), player.rating_sum.into());
    let average_rating = (player.rated_appearances > 0)
        .then(|| player.rating_sum / f64::from(player.rated_appearances));
    fields.insert("average_rating".into(), average_rating.into());
    for stat in SeasonStat::ALL {
        fields.insert(stat.name().into(), player.total(stat).into());
    }
    Value::Object(fields)
}

#[test]
#[ignore]
fn save_stats_reference_harness() {
    let path = std::env::var_os("FM_VALUESCOUT_SAVE_FILE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_file() && std::fs::File::open(path).is_ok())
        .expect("private save file is missing or unreadable");
    let started = Instant::now();
    let stats = read_season_stats(&path)
        .unwrap_or_else(|error| panic!("save statistics read failed: {}", error.kind()));
    let elapsed_ms = started.elapsed().as_millis();

    let players: Map<String, Value> = stats
        .players
        .iter()
        .map(|player| (player.unique_id.to_string(), player_report(player)))
        .collect();
    let report = json!({
        "test": "save_stats_reference_harness",
        "host": {
            "os": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "execution_context": if cfg!(target_os = "windows") {
                "native-windows"
            } else if std::env::var_os("WSL_INTEROP").is_some()
                || std::fs::read_to_string("/proc/version").is_ok_and(|v| v.contains("Microsoft"))
            {
                "wsl"
            } else {
                "native-unix"
            },
            "filesystem_context": if cfg!(target_os = "windows") {
                "windows-filesystem"
            } else if path.to_string_lossy().starts_with("/mnt/") {
                "mounted-windows-filesystem"
            } else {
                "native-unix-filesystem"
            },
            "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        },
        "build": stats.info.build,
        "known_build": stats.info.known_build,
        "game_date": stats.info.game_date,
        "elapsed_ms": elapsed_ms,
        "cohort_size": stats.players.len(),
        "unique_players": players.len(),
        "shortlists": match &stats.shortlists {
            Ok(shortlists) => shortlists
                .iter()
                .map(|shortlist| json!({
                    "name": shortlist.name,
                    "players": shortlist.player_unique_ids.len(),
                }))
                .collect(),
            Err(error) => json!({ "error": error.kind() }),
        },
        "staff_shortlist": match &stats.staff_shortlist {
            Ok(staff) => staff
                .iter()
                .map(|person| json!({
                    "unique_id": person.unique_id,
                    "preferred_job": person.preferred_job,
                    "coaching_qualifications": person.coaching_qualifications,
                }))
                .collect(),
            Err(error) => json!({ "error": error.kind() }),
        },
        "players": players,
    });
    let json = serde_json::to_string(&report).expect("reference report serializes");
    assert!(!json.contains(path.to_string_lossy().as_ref()));
    println!("{json}");
}
