use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::features::player::boost_gate;
use crate::features::player::commands::{
    self as player_commands, SquadPlayerBoostProgressDto, SquadPlayerBoostResultDto,
};
use crate::features::player::service::PlayerBoostError;
use crate::features::staff::commands::{
    self as staff_commands, MyStaffBoostProgressDto, MyStaffBoostResultDto,
};
use crate::features::staff::service::StaffBoostError;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManagedClubBoostPhase {
    Wonderkids,
    PlayerCurrentAbility,
    StaffCurrentAbility,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedClubBoostProgressDto {
    pub phase: ManagedClubBoostPhase,
    pub processed: usize,
    pub total: usize,
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedClubBoostResultDto {
    pub wonderkids: SquadPlayerBoostResultDto,
    pub player_current_ability: Option<SquadPlayerBoostResultDto>,
    pub staff_current_ability: Option<MyStaffBoostResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ManagedClubBoostError {
    Player(PlayerBoostError),
    Staff(StaffBoostError),
}

impl From<PlayerBoostError> for ManagedClubBoostError {
    fn from(error: PlayerBoostError) -> Self {
        Self::Player(error)
    }
}

impl From<StaffBoostError> for ManagedClubBoostError {
    fn from(error: StaffBoostError) -> Self {
        Self::Staff(error)
    }
}

#[tauri::command]
pub fn boost_managed_club(
    db: State<'_, Db>,
    on_progress: tauri::ipc::Channel<ManagedClubBoostProgressDto>,
) -> Result<ManagedClubBoostResultDto, ManagedClubBoostError> {
    let guard = boost_gate::acquire_boost_gate().map_err(|message| PlayerBoostError::Bridge {
        kind: "inProgress".to_string(),
        message,
    })?;

    execute_managed_club_boost(
        || {
            player_commands::boost_squad_wonderkid_mentality_under_guard(
                db.inner(),
                &guard,
                |progress| {
                    send_player_progress(&on_progress, ManagedClubBoostPhase::Wonderkids, progress)
                },
            )
            .map_err(Into::into)
        },
        || {
            player_commands::boost_squad_current_ability_under_guard(
                db.inner(),
                &guard,
                |progress| {
                    send_player_progress(
                        &on_progress,
                        ManagedClubBoostPhase::PlayerCurrentAbility,
                        progress,
                    )
                },
            )
            .map_err(Into::into)
        },
        || {
            staff_commands::boost_my_staff_current_ability_under_guard(
                db.inner(),
                &guard,
                |progress| send_staff_progress(&on_progress, progress),
            )
            .map_err(Into::into)
        },
    )
}

fn execute_managed_club_boost<W, P, S>(
    boost_wonderkids: W,
    boost_player_current_ability: P,
    boost_staff_current_ability: S,
) -> Result<ManagedClubBoostResultDto, ManagedClubBoostError>
where
    W: FnOnce() -> Result<SquadPlayerBoostResultDto, ManagedClubBoostError>,
    P: FnOnce() -> Result<SquadPlayerBoostResultDto, ManagedClubBoostError>,
    S: FnOnce() -> Result<MyStaffBoostResultDto, ManagedClubBoostError>,
{
    let wonderkids = boost_wonderkids()?;
    if wonderkids.recovery_required {
        return Ok(ManagedClubBoostResultDto {
            wonderkids,
            player_current_ability: None,
            staff_current_ability: None,
        });
    }

    let player_current_ability = boost_player_current_ability()?;
    if player_current_ability.recovery_required {
        return Ok(ManagedClubBoostResultDto {
            wonderkids,
            player_current_ability: Some(player_current_ability),
            staff_current_ability: None,
        });
    }

    Ok(ManagedClubBoostResultDto {
        wonderkids,
        player_current_ability: Some(player_current_ability),
        staff_current_ability: Some(boost_staff_current_ability()?),
    })
}

fn send_player_progress(
    channel: &tauri::ipc::Channel<ManagedClubBoostProgressDto>,
    phase: ManagedClubBoostPhase,
    progress: SquadPlayerBoostProgressDto,
) -> bool {
    send_progress(
        channel,
        ManagedClubBoostProgressDto {
            phase,
            processed: progress.processed,
            total: progress.total,
            updated: progress.updated,
            skipped: progress.skipped,
            failed: progress.failed,
        },
    )
}

fn send_staff_progress(
    channel: &tauri::ipc::Channel<ManagedClubBoostProgressDto>,
    progress: MyStaffBoostProgressDto,
) -> bool {
    send_progress(
        channel,
        ManagedClubBoostProgressDto {
            phase: ManagedClubBoostPhase::StaffCurrentAbility,
            processed: progress.processed,
            total: progress.total,
            updated: progress.updated,
            skipped: progress.skipped,
            failed: progress.failed,
        },
    )
}

fn send_progress(
    channel: &tauri::ipc::Channel<ManagedClubBoostProgressDto>,
    progress: ManagedClubBoostProgressDto,
) -> bool {
    match channel.send(progress) {
        Ok(()) => true,
        Err(error) => {
            log::debug!("managed-club boost progress delivery failed: {error}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    fn player_result(recovery_required: bool) -> SquadPlayerBoostResultDto {
        SquadPlayerBoostResultDto {
            updated: 1,
            skipped: 0,
            failed: 0,
            recovery_required,
            recovery_message: recovery_required.then(|| "Load Data again".to_string()),
        }
    }

    fn staff_result() -> MyStaffBoostResultDto {
        MyStaffBoostResultDto {
            updated: 1,
            skipped: 0,
            failed: 0,
            recovery_required: false,
            recovery_message: None,
        }
    }

    #[test]
    fn runs_all_three_boosts_in_order() {
        let phases = RefCell::new(Vec::new());

        let result = execute_managed_club_boost(
            || {
                phases.borrow_mut().push("wonderkids");
                Ok(player_result(false))
            },
            || {
                phases.borrow_mut().push("player CA");
                Ok(player_result(false))
            },
            || {
                phases.borrow_mut().push("staff CA");
                Ok(staff_result())
            },
        )
        .expect("combined boost");

        assert_eq!(phases.into_inner(), ["wonderkids", "player CA", "staff CA"]);
        assert!(result.player_current_ability.is_some());
        assert!(result.staff_current_ability.is_some());
    }

    #[test]
    fn stops_after_wonderkids_require_load_data() {
        let result = execute_managed_club_boost(
            || Ok(player_result(true)),
            || panic!("player CA must not run"),
            || panic!("staff CA must not run"),
        )
        .expect("partial combined boost");

        assert!(result.wonderkids.recovery_required);
        assert!(result.player_current_ability.is_none());
        assert!(result.staff_current_ability.is_none());
    }

    #[test]
    fn stops_after_player_ca_requires_load_data() {
        let result = execute_managed_club_boost(
            || Ok(player_result(false)),
            || Ok(player_result(true)),
            || panic!("staff CA must not run"),
        )
        .expect("partial combined boost");

        assert!(result
            .player_current_ability
            .is_some_and(|phase| phase.recovery_required));
        assert!(result.staff_current_ability.is_none());
    }

    #[test]
    fn serializes_the_combined_result_for_the_frontend_contract() {
        let result = ManagedClubBoostResultDto {
            wonderkids: player_result(false),
            player_current_ability: Some(player_result(false)),
            staff_current_ability: Some(staff_result()),
        };

        let json = serde_json::to_value(result).expect("serialize result");

        assert_eq!(json["wonderkids"]["updated"], 1);
        assert_eq!(json["playerCurrentAbility"]["updated"], 1);
        assert_eq!(json["staffCurrentAbility"]["updated"], 1);
    }
}
