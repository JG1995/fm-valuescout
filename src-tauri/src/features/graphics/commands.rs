use super::faces::GameFaces;
use super::index::GraphicsKind;
use super::runtime::{picker, GraphicsRuntime, GraphicsStatus, ImageLookupResult};
use crate::blocking;
use crate::db::Db;
use serde::{Deserialize, Serialize};
use tauri::{http, AppHandle, Manager, State};

#[tauri::command(async)]
pub fn get_graphics_status(
    runtime: State<'_, GraphicsRuntime>,
    faces: State<'_, GameFaces>,
) -> GraphicsStatus {
    status(&runtime, &faces)
}

fn status(runtime: &GraphicsRuntime, faces: &GameFaces) -> GraphicsStatus {
    GraphicsStatus {
        game_faces: faces.available(),
        ..runtime.status()
    }
}

fn app_status(app: &AppHandle) -> GraphicsStatus {
    status(&app.state::<GraphicsRuntime>(), &app.state::<GameFaces>())
}

#[tauri::command]
pub async fn choose_graphics_root(
    app: AppHandle,
    db: State<'_, Db>,
) -> Result<GraphicsStatus, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let runtime = app.state::<GraphicsRuntime>();
        let Some(root) = picker(&app)? else {
            return Ok(app_status(&app));
        };
        if root.as_os_str().is_empty() {
            return Ok(app_status(&app));
        }
        if let Some(target) = runtime.persist_transition(&db.0, Some(root))? {
            runtime.enqueue(target);
        }
        Ok(app_status(&app))
    })
    .await
}

#[tauri::command]
pub async fn clear_graphics_root(
    app: AppHandle,
    db: State<'_, Db>,
) -> Result<GraphicsStatus, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let runtime = app.state::<GraphicsRuntime>();
        if runtime.persist_transition(&db.0, None)?.is_some() {
            runtime.cancel_pending();
        }
        Ok(app_status(&app))
    })
    .await
}

#[tauri::command]
pub async fn rescan_graphics(app: AppHandle, db: State<'_, Db>) -> Result<GraphicsStatus, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let runtime = app.state::<GraphicsRuntime>();
        if let Some(target) = runtime.begin_rescan(&db.0)? {
            runtime.enqueue(target);
        }
        Ok(app_status(&app))
    })
    .await
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolRequest {
    pub generation: u64,
    pub kind: GraphicsKind,
    pub uid: u32,
}

/// A request for FM's own face of one player of one snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameFaceRequest {
    pub snapshot_id: i64,
    pub uid: u32,
}

/// The three path segments of a request in the closed grammar `/<number>/<kind>/<uid>`.
fn identity_segments(request: &http::Request<Vec<u8>>) -> Result<[&str; 3], http::StatusCode> {
    if request.method() != http::Method::GET {
        return Err(http::StatusCode::METHOD_NOT_ALLOWED);
    }
    let uri = request.uri();
    // URI fragments are browser-side and are not transmitted to this handler;
    // they are therefore outside the handler grammar.
    if uri.scheme_str() != Some("graphics")
        || uri.host() != Some("localhost")
        || uri.port().is_some()
        || uri.query().is_some()
    {
        return Err(http::StatusCode::BAD_REQUEST);
    }
    let mut segments = uri.path().split('/');
    match (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) {
        (Some(""), Some(number), Some(kind), Some(uid), None) => Ok([number, kind, uid]),
        _ => Err(http::StatusCode::BAD_REQUEST),
    }
}

fn parse_uid(segment: &str) -> Result<u32, http::StatusCode> {
    Some(segment)
        .filter(|segment| canonical_decimal(segment, false))
        .and_then(|segment| segment.parse().ok())
        .filter(|uid: &u32| *uid > 0)
        .ok_or(http::StatusCode::BAD_REQUEST)
}

pub fn parse_protocol_request(
    request: &http::Request<Vec<u8>>,
) -> Result<ProtocolRequest, http::StatusCode> {
    let [generation, kind, uid] = identity_segments(request)?;
    let generation = Some(generation)
        .filter(|segment| canonical_decimal(segment, true))
        .and_then(|segment| segment.parse().ok())
        .ok_or(http::StatusCode::BAD_REQUEST)?;
    let kind = match kind {
        "personPortrait" => GraphicsKind::PersonPortrait,
        "clubLogo" => GraphicsKind::ClubLogo,
        "clubIcon" => GraphicsKind::ClubIcon,
        _ => return Err(http::StatusCode::BAD_REQUEST),
    };
    Ok(ProtocolRequest {
        generation,
        kind,
        uid: parse_uid(uid)?,
    })
}

/// Parses `/<snapshot id>/gameFace/<uid>`; `None` for every other request.
pub fn parse_game_face_request(request: &http::Request<Vec<u8>>) -> Option<GameFaceRequest> {
    let [snapshot_id, "gameFace", uid] = identity_segments(request).ok()? else {
        return None;
    };
    let snapshot_id = Some(snapshot_id)
        .filter(|segment| canonical_decimal(segment, false))
        .and_then(|segment| segment.parse().ok())
        .filter(|snapshot_id: &i64| *snapshot_id > 0)?;
    Some(GameFaceRequest {
        snapshot_id,
        uid: parse_uid(uid).ok()?,
    })
}

fn canonical_decimal(segment: &str, allow_zero: bool) -> bool {
    !segment.is_empty()
        && segment.bytes().all(|byte| byte.is_ascii_digit())
        && (segment == "0" || !segment.starts_with('0'))
        && (allow_zero || segment != "0")
}

fn protocol_response(status: http::StatusCode) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Vec::new())
        .expect("static protocol response is valid")
}

fn image_protocol_response(result: ImageLookupResult) -> http::Response<Vec<u8>> {
    match result {
        ImageLookupResult::Available { bytes, mime } => http::Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, mime)
            .header(http::header::X_CONTENT_TYPE_OPTIONS, "nosniff")
            .header(http::header::CACHE_CONTROL, "no-store")
            .body(bytes)
            .expect("static protocol response is valid"),
        ImageLookupResult::Missing => protocol_response(http::StatusCode::NOT_FOUND),
    }
}

/// Answers a request of the `graphics` protocol: a game face from the snapshot's stored
/// appearance and FM's portrait bundle, or an image of the selected graphics folder.
pub fn respond(
    request: http::Request<Vec<u8>>,
    runtime: &GraphicsRuntime,
    faces: &GameFaces,
    db: &Db,
) -> http::Response<Vec<u8>> {
    let Some(face) = parse_game_face_request(&request) else {
        return graphics_protocol_response(request, runtime);
    };
    let result = match db.read() {
        Ok(conn) => faces.resolve(&conn, face.snapshot_id, face.uid),
        Err(_) => ImageLookupResult::Missing,
    };
    image_protocol_response(result)
}

pub fn graphics_protocol_response(
    request: http::Request<Vec<u8>>,
    runtime: &GraphicsRuntime,
) -> http::Response<Vec<u8>> {
    let parsed = match parse_protocol_request(&request) {
        Ok(parsed) => parsed,
        Err(status) => return protocol_response(status),
    };
    image_protocol_response(runtime.resolve_at_generation(
        parsed.generation,
        parsed.kind,
        parsed.uid,
    ))
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GraphicsKindDto {
    PersonPortrait,
    ClubLogo,
    ClubIcon,
}
impl From<GraphicsKindDto> for GraphicsKind {
    fn from(v: GraphicsKindDto) -> Self {
        match v {
            GraphicsKindDto::PersonPortrait => Self::PersonPortrait,
            GraphicsKindDto::ClubLogo => Self::ClubLogo,
            GraphicsKindDto::ClubIcon => Self::ClubIcon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: http::Method, uri: &str) -> http::Request<Vec<u8>> {
        http::Request::builder()
            .method(method)
            .uri(uri)
            .body(Vec::new())
            .unwrap()
    }

    #[test]
    fn a_game_face_request_names_one_snapshot_and_one_player() {
        let parse = |method, uri| parse_game_face_request(&request(method, uri));

        assert_eq!(
            parse(
                http::Method::GET,
                "graphics://localhost/12/gameFace/2002086752"
            ),
            Some(GameFaceRequest {
                snapshot_id: 12,
                uid: 2_002_086_752,
            })
        );
        for uri in [
            "graphics://localhost/0/gameFace/77",
            "graphics://localhost/012/gameFace/77",
            "graphics://localhost/-1/gameFace/77",
            "graphics://localhost/12/gameFace/0",
            "graphics://localhost/12/gameFace/4294967296",
            "graphics://localhost/12/gameFace/77/extra",
            "graphics://localhost/12/gameFace/77?size=2",
            "graphics://localhost/12/gameface/77",
            "graphics://localhost/12/personPortrait/77",
            "graphics://elsewhere/12/gameFace/77",
            "graphics://localhost:80/12/gameFace/77",
        ] {
            assert_eq!(parse(http::Method::GET, uri), None, "{uri}");
        }
        assert_eq!(
            parse(http::Method::POST, "graphics://localhost/12/gameFace/77"),
            None
        );
        // The graphics folder's grammar does not accept the kind either.
        assert_eq!(
            parse_protocol_request(&request(
                http::Method::GET,
                "graphics://localhost/12/gameFace/77"
            )),
            Err(http::StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn protocol_parser_accepts_only_canonical_identity() {
        assert_eq!(
            parse_protocol_request(&request(
                http::Method::GET,
                "graphics://localhost/12/personPortrait/42"
            )),
            Ok(ProtocolRequest {
                generation: 12,
                kind: GraphicsKind::PersonPortrait,
                uid: 42,
            })
        );
    }

    #[test]
    fn protocol_parser_rejects_every_closed_grammar_escape() {
        for (method, uri) in [
            (
                http::Method::POST,
                "graphics://localhost/12/personPortrait/42",
            ),
            (http::Method::GET, "graphics://other/12/personPortrait/42"),
            (
                http::Method::GET,
                "http://graphics.localhost/12/personPortrait/42",
            ),
            (http::Method::GET, "graphics://localhost//personPortrait/42"),
            (http::Method::GET, "graphics://localhost/12/personPortrait/"),
            (
                http::Method::GET,
                "graphics://localhost/12/personPortrait/42/extra",
            ),
            (http::Method::GET, "graphics://localhost/12/unknown/42"),
            (
                http::Method::GET,
                "graphics://localhost/12/personPortrait/0",
            ),
            (
                http::Method::GET,
                "graphics://localhost/12/personPortrait/-1",
            ),
            (
                http::Method::GET,
                "graphics://localhost/12/personPortrait/4.2",
            ),
            (
                http::Method::GET,
                "graphics://localhost/012/personPortrait/42",
            ),
            (
                http::Method::GET,
                "graphics://localhost/12/personPortrait/042",
            ),
            (
                http::Method::GET,
                "graphics://localhost:80/12/personPortrait/42",
            ),
            (
                http::Method::GET,
                "graphics://localhost:8080/12/personPortrait/42",
            ),
            (
                http::Method::GET,
                "graphics://localhost/12/personPortrait/42?x=1",
            ),
        ] {
            assert!(
                parse_protocol_request(&request(method, uri)).is_err(),
                "{uri}"
            );
        }
    }

    #[test]
    fn image_response_is_raw_and_security_bounded() {
        let response = image_protocol_response(ImageLookupResult::Available {
            bytes: vec![1, 2, 3],
            mime: "image/webp",
        });
        assert_eq!(response.status(), http::StatusCode::OK);
        assert_eq!(response.body(), &[1, 2, 3]);
        assert_eq!(response.headers()[http::header::CONTENT_TYPE], "image/webp");
        assert_eq!(
            response.headers()[http::header::X_CONTENT_TYPE_OPTIONS],
            "nosniff"
        );
        assert_eq!(response.headers()[http::header::CACHE_CONTROL], "no-store");
    }

    #[test]
    fn missing_response_is_bounded_non_success() {
        let response = image_protocol_response(ImageLookupResult::Missing);
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
        assert!(response.body().is_empty());
    }
}
