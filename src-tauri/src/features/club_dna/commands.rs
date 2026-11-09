use serde::Serialize;
use tauri::State;

use crate::blocking;
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
pub async fn get_club_dna(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<Option<ClubDnaDefinitionDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || read_club_dna(&db, save_id, &context_token)).await
}

fn read_club_dna(
    db: &Db,
    save_id: i64,
    context_token: &str,
) -> Result<Option<ClubDnaDefinitionDto>, String> {
    let conn = db.read()?;
    service::get_club_dna(&conn, save_id, context_token)
        .map(|definition| definition.map(Into::into))
}

#[tauri::command]
pub async fn set_club_dna(
    save_id: i64,
    context_token: String,
    attribute_ids: Vec<String>,
    db: State<'_, Db>,
) -> Result<ClubDnaUpsertResultDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        service::set_club_dna(&conn, save_id, &context_token, attribute_ids).map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn remove_club_dna(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<ClubDnaRemoveResultDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        service::remove_club_dna(&conn, save_id, &context_token)
            .map(|removed| ClubDnaRemoveResultDto { removed })
    })
    .await
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

    #[test]
    fn get_club_dna_answers_on_a_reader() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        let (save_id, context_token): (i64, String) =
            db.0.lock()
                .expect("lock writer")
                .query_row(
                    "SELECT id, context_token FROM saves WHERE is_active = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .expect("active save");
        let token = context_token.clone();
        let (absent, stale) = crate::db::read_while_writer_is_held(&db, move |db| {
            (
                read_club_dna(db, save_id, &token),
                read_club_dna(db, save_id, "stale-token"),
            )
        });
        assert!(absent.expect("absent definition on a reader").is_none());
        assert_eq!(
            stale.err().as_deref(),
            Some("Save changed or is no longer active")
        );

        service::set_club_dna(
            &db.0.lock().expect("lock writer"),
            save_id,
            &context_token,
            vec!["attr.Acceleration".to_string()],
        )
        .expect("store definition");
        let stored = crate::db::read_while_writer_is_held(&db, move |db| {
            read_club_dna(db, save_id, &context_token)
        });
        assert_eq!(
            stored
                .expect("stored definition on a reader")
                .map(|definition| definition.attribute_ids),
            Some(vec!["attr.Acceleration".to_string()])
        );
    }
}
