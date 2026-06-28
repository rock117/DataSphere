use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "klines")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub code: String,
    pub date: chrono::NaiveDate,
    pub open: Decimal,
    pub close: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub volume: i64,
    pub amount: Decimal,
    pub turnover: Option<Decimal>,
    pub pct_change: Option<Decimal>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
