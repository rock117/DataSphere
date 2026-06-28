use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "task_runs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub task_id: i64,
    pub status: String,
    pub trigger_type: String,
    pub started_at: chrono::NaiveDateTime,
    pub finished_at: Option<chrono::NaiveDateTime>,
    pub records_affected: i64,
    pub error: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
