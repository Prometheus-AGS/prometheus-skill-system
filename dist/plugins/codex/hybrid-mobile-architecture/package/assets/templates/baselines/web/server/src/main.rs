// TJ-ARCH-MOB-001 compliant
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use gen_ui_notes::{Note, NoteError};
use std::path::PathBuf;
#[derive(Clone)]
struct AppState {
    database: PathBuf,
}
#[derive(serde::Deserialize)]
struct CreateNote {
    title: String,
}
struct ApiError(NoteError);
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self.0 {
            NoteError::InvalidTitle => {
                (StatusCode::BAD_REQUEST, self.0.to_string()).into_response()
            }
            NoteError::Storage(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not access note storage",
            )
                .into_response(),
        }
    }
}
async fn list(State(state): State<AppState>) -> Result<Json<Vec<Note>>, ApiError> {
    tokio::task::spawn_blocking(move || gen_ui_notes::open(&state.database)?.list())
        .await
        .map_err(|error| ApiError(NoteError::Storage(error.to_string())))?
        .map(Json)
        .map_err(ApiError)
}
async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateNote>,
) -> Result<(StatusCode, Json<Note>), ApiError> {
    tokio::task::spawn_blocking(move || gen_ui_notes::open(&state.database)?.create(&input.title))
        .await
        .map_err(|error| ApiError(NoteError::Storage(error.to_string())))?
        .map(|note| (StatusCode::CREATED, Json(note)))
        .map_err(ApiError)
}
async fn capabilities() -> Result<Json<Vec<gen_ui_notes::Capability>>, ApiError> {
    let mut registry = gen_ui_notes::capabilities().map_err(ApiError)?;
    registry.extend(
        gen_ui_notes::parse_capabilities(include_str!("../capabilities/index.json"))
            .map_err(ApiError)?,
    );
    Ok(Json(registry))
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = PathBuf::from(std::env::var_os("APP_DATA_DIR").unwrap_or_else(|| ".data".into()));
    let state = AppState {
        database: data.join("notes.sqlite3"),
    };
    gen_ui_notes::open(&state.database)?;
    let app = Router::new()
        .route("/api/notes", get(list).post(create))
        .route("/api/capabilities", get(capabilities))
        .route(
            "/api/runtime",
            get(|| async {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "UAR runtime is not configured",
                )
            }),
        )
        .fallback_service(tower_http::services::ServeDir::new(
            std::env::var("WEB_DIST_DIR").unwrap_or_else(|_| "web/dist".to_owned()),
        ))
        .with_state(state);
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_owned())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    println!("Notes ready at http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
