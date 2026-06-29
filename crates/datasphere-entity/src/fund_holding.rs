use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize)]
#[sea_orm(table_name = "fund_holdings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub fund_code: String,
    pub stock_code: String,
    pub stock_name: String,
    pub report_date: chrono::NaiveDate,
    pub weight: Decimal,
    pub shares: Option<i64>,
    pub market_value: Option<Decimal>,
    pub rank: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
