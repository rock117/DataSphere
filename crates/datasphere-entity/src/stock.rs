use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "stocks")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub code: String,
    pub symbol: String,
    pub name: String,
    pub market: String,
    pub exchange: String,
    pub list_date: Option<chrono::NaiveDate>,
    pub delist_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
