use datasphere_core::domain::StockQuote;
use datasphere_entity::stock;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    Set,
};

pub struct StockService;

impl StockService {
    /// 按 code upsert 单条股票（存在则更新，不存在则插入）
    pub async fn upsert(db: &DatabaseConnection, q: &StockQuote) -> anyhow::Result<()> {
        let existing = stock::Entity::find()
            .filter(stock::Column::Code.eq(&q.code))
            .one(db)
            .await?;

        if let Some(m) = existing {
            let mut am: stock::ActiveModel = m.into();
            am.symbol = Set(q.symbol.clone());
            am.name = Set(q.name.clone());
            am.market = Set(q.market.to_string());
            am.exchange = Set(q.exchange.clone());
            am.list_date = Set(q.list_date);
            am.delist_date = Set(q.delist_date);
            am.update(db).await?;
        } else {
            let am = stock::ActiveModel {
                code: Set(q.code.clone()),
                symbol: Set(q.symbol.clone()),
                name: Set(q.name.clone()),
                market: Set(q.market.to_string()),
                exchange: Set(q.exchange.clone()),
                list_date: Set(q.list_date),
                delist_date: Set(q.delist_date),
                ..Default::default()
            };
            am.insert(db).await?;
        }
        Ok(())
    }

    /// 批量 upsert
    pub async fn upsert_many(
        db: &DatabaseConnection,
        quotes: &[StockQuote],
    ) -> anyhow::Result<usize> {
        let mut count = 0;
        for q in quotes {
            Self::upsert(db, q).await?;
            count += 1;
        }
        Ok(count)
    }

    /// 获取全市场股票代码列表
    pub async fn list_all_codes(db: &DatabaseConnection) -> anyhow::Result<Vec<String>> {
        let rows = stock::Entity::find()
            .order_by_asc(stock::Column::Code)
            .all(db)
            .await?;
        Ok(rows.into_iter().map(|m| m.code).collect())
    }

    /// 分页查询
    pub async fn paginate(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        q: Option<&str>,
    ) -> anyhow::Result<(Vec<stock::Model>, u64)> {
        let mut query = stock::Entity::find();
        if let Some(q) = q {
            if !q.is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(stock::Column::Code.contains(q))
                        .add(stock::Column::Name.contains(q)),
                );
            }
        }
        let total = query.clone().count(db).await?;
        let rows = query
            .order_by_asc(stock::Column::Code)
            .offset(Some((page - 1) * per_page))
            .limit(per_page)
            .all(db)
            .await?;
        Ok((rows, total))
    }

    /// 按代码查单只
    pub async fn find_by_code(
        db: &DatabaseConnection,
        code: &str,
    ) -> anyhow::Result<Option<stock::Model>> {
        stock::Entity::find()
            .filter(stock::Column::Code.eq(code))
            .one(db)
            .await
            .map_err(Into::into)
    }
}
