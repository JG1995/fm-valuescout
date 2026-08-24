use serde::Serialize;
use tauri::State;

use crate::db::Db;

use super::service::{self, ClubDnaDefinition, ClubDnaUpsertResult};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClubDnaDefinitionDto {
    pub attribute_ids: Vec<String>,
}

impl From<ClubDnaDefinition> for ClubDnaDefinitionDto {
    fn from(definition: ClubDnaDefinition) -> Self {
        Self {
            attribute_ids: definition.attribute_ids,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClubDnaUpsertResultDto {
    pub definition: ClubDnaDefinitionDto,
    pub created: bool,
}

impl From<ClubDnaUpsertResult> for ClubDnaUpsertResultDto {
    fn from(result: ClubDnaUpsertResult) -> Self {
        Self {
            definition: result.definition.into(),
            created: result.created,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClubDnaRemoveResultDto {
    pub removed: bool,
}

#[tauri::command]
pub fn get_club_dna(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<Option<ClubDnaDefinitionDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    service::get_club_dna(&conn, save_id, &context_token)
        .map(|definition| definition.map(Into::into))
}

#[tauri::command]
pub fn set_club_dna(
    save_id: i64,
    context_token: String,
    attribute_ids: Vec<String>,
    db: State<'_, Db>,
) -> Result<ClubDnaUpsertResultDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    service::set_club_dna(&conn, save_id, &context_token, attribute_ids).map(Into::into)
}

#[tauri::command]
pub fn remove_club_dna(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<ClubDnaRemoveResultDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    service::remove_club_dna(&conn, save_id, &context_token)
        .map(|removed| ClubDnaRemoveResultDto { removed })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_dtos_use_camel_case_and_return_mutation_state() {
        let definition = ClubDnaDefinitionDto {
            attribute_ids: vec!["attr.Acceleration".to_string()],
        };
        let value = serde_json::to_value(ClubDnaUpsertResultDto {
            definition,
            created: true,
        })
        .expect("serialize upsert result");
        assert_eq!(value["definition"]["attributeIds"][0], "attr.Acceleration");
        assert_eq!(value["created"], true);
        assert_eq!(
            serde_json::to_value(ClubDnaRemoveResultDto { removed: false })
                .expect("serialize removal result")["removed"],
            false
        );
    }
}
