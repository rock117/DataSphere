use datasphere_core::domain::KlineQuote;
use datasphere_entity::kline;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

pub struct KlineService;

impl KlineService {
    /// 按 (code, date) upsert 单条日K（存在则更新，不存在则插入）
    pub async fn upsert(db: &DatabaseConnection, k: &KlineQuote) -> anyhow::Result<()> {
        let existing = kline::Entity::find()
            .filter(kline::Column::Code.eq(&k.code))
            .filter(kline::Column::Date.eq(k.date))
            .one(db)
            .await?;

        if let Some(m) = existing {
            let mut am: kline::ActiveModel = m.into();
            am.open = Set(k.open);
            am.close = Set(k.close);
            am.high = Set(k.high);
            am.low = Set(k.low);
            am.volume = Set(k.volume);
            am.amount = Set(k.amount);
            am.turnover = Set(k.turnover);
            am.pct_change = Set(k.pct_change);
            am.update(db).await?;
        } else {
            let am = kline::ActiveModel {
                code: Set(k.code.clone()),
                date: Set(k.date),
                open: Set(k.open),
                close: Set(k.close),
                high: Set(k.high),
                low: Set(k.low),
                volume: Set(k.volume),
                amount: Set(k.amount),
                turnover: Set(k.turnover),
                pct_change: Set(k.pct_change),
                ..Default::default()
            };
            am.insert(db).await?;
        }
        Ok(())
    }

    /// 批量 upsert
    pub async fn upsert_many(
        db: &DatabaseConnection,
        quotes: &[KlineQuote],
    ) -> anyhow::Result<usize> {
        let mut count = 0;
        for k in quotes {
            Self::upsert(db, k).await?;
            count += 1;
        }
        Ok(count)
    }

    /// 查询某股票在日期范围内的日K
    pub async fn query(
        db: &DatabaseConnection,
        code: &str,
        start: Option<chrono::NaiveDate>,
        end: Option<chrono::NaiveDate>,
    ) -> anyhow::Result<Vec<kline::Model>> {
        let mut query = kline::Entity::find().filter(kline::Column::Code.eq(code));
        if let Some(s) = start {
            query = query.filter(kline::Column::Date.gte(s));
        }
        if let Some(e) = end {
            query = query.filter(kline::Column::Date.lte(e));
        }
        let rows = query.order_by_asc(kline::Column::Date).all(db).await?;
        Ok(rows)
    }
}
