use datasphere_core::domain::FundHolding;
use datasphere_entity::fund_holding;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};

pub struct FundHoldingService;

fn to_decimal(v: f64) -> Decimal {
    Decimal::try_from(v).unwrap_or_default()
}

impl FundHoldingService {
    /// 按 (fund_code, stock_code, report_date) upsert 单条
    pub async fn upsert(db: &DatabaseConnection, h: &FundHolding) -> anyhow::Result<()> {
        let existing = fund_holding::Entity::find()
            .filter(fund_holding::Column::FundCode.eq(&h.fund_code))
            .filter(fund_holding::Column::StockCode.eq(&h.stock_code))
            .filter(fund_holding::Column::ReportDate.eq(h.report_date))
            .one(db)
            .await?;

        if let Some(m) = existing {
            let mut am: fund_holding::ActiveModel = m.into();
            am.stock_name = Set(h.stock_name.clone());
            am.weight = Set(to_decimal(h.weight));
            am.shares = Set(h.shares);
            am.market_value = Set(h.market_value.map(to_decimal));
            am.rank = Set(h.rank);
            am.update(db).await?;
        } else {
            let am = fund_holding::ActiveModel {
                fund_code: Set(h.fund_code.clone()),
                stock_code: Set(h.stock_code.clone()),
                stock_name: Set(h.stock_name.clone()),
                report_date: Set(h.report_date),
                weight: Set(to_decimal(h.weight)),
                shares: Set(h.shares),
                market_value: Set(h.market_value.map(to_decimal)),
                rank: Set(h.rank),
                ..Default::default()
            };
            am.insert(db).await?;
        }
        Ok(())
    }

    /// 批量 upsert
    pub async fn upsert_many(
        db: &DatabaseConnection,
        holdings: &[FundHolding],
    ) -> anyhow::Result<usize> {
        let mut count = 0;
        for h in holdings {
            Self::upsert(db, h).await?;
            count += 1;
        }
        Ok(count)
    }

    /// 查询某基金的成分股（按报告期倒序，再按排名正序）
    pub async fn list_by_fund(
        db: &DatabaseConnection,
        fund_code: &str,
        limit: u64,
    ) -> anyhow::Result<Vec<fund_holding::Model>> {
        fund_holding::Entity::find()
            .filter(fund_holding::Column::FundCode.eq(fund_code))
            .order_by_desc(fund_holding::Column::ReportDate)
            .order_by_asc(fund_holding::Column::Rank)
            .limit(limit)
            .all(db)
            .await
            .map_err(Into::into)
    }

    /// 查询某基金指定报告期的成分股
    pub async fn list_by_fund_and_date(
        db: &DatabaseConnection,
        fund_code: &str,
        report_date: chrono::NaiveDate,
    ) -> anyhow::Result<Vec<fund_holding::Model>> {
        fund_holding::Entity::find()
            .filter(fund_holding::Column::FundCode.eq(fund_code))
            .filter(fund_holding::Column::ReportDate.eq(report_date))
            .order_by_asc(fund_holding::Column::Rank)
            .all(db)
            .await
            .map_err(Into::into)
    }

    /// 获取某基金所有报告期列表
    pub async fn list_report_dates(
        db: &DatabaseConnection,
        fund_code: &str,
    ) -> anyhow::Result<Vec<chrono::NaiveDate>> {
        let rows = fund_holding::Entity::find()
            .filter(fund_holding::Column::FundCode.eq(fund_code))
            .order_by_desc(fund_holding::Column::ReportDate)
            .all(db)
            .await?;
        let mut dates: Vec<chrono::NaiveDate> = rows.into_iter().map(|m| m.report_date).collect();
        dates.dedup();
        Ok(dates)
    }
}
