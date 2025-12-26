use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: Option<PgPool>,
}
