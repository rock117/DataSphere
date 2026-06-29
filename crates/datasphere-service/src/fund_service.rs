use datasphere_core::domain::FundQuote;
use datasphere_entity::fund;
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    Set,
};

pub struct FundService;

fn to_decimal(v: f64) -> Decimal {
    Decimal::try_from(v).unwrap_or_default()
}

impl FundService {
    /// 按 code upsert 单条基金
    pub async fn upsert(db: &DatabaseConnection, q: &FundQuote) -> anyhow::Result<()> {
        let existing = fund::Entity::find()
            .filter(fund::Column::Code.eq(&q.code))
            .one(db)
            .await?;

        if let Some(m) = existing {
            let mut am: fund::ActiveModel = m.into();
            am.name = Set(q.name.clone());
            am.fund_type = Set(q.fund_type.to_string());
            am.management = Set(q.management.clone());
            am.custodian = Set(q.custodian.clone());
            am.setup_date = Set(q.setup_date);
            am.latest_scale = Set(q.latest_scale.map(to_decimal));
            am.update(db).await?;
        } else {
            let am = fund::ActiveModel {
                code: Set(q.code.clone()),
                name: Set(q.name.clone()),
                fund_type: Set(q.fund_type.to_string()),
                management: Set(q.management.clone()),
                custodian: Set(q.custodian.clone()),
                setup_date: Set(q.setup_date),
                latest_scale: Set(q.latest_scale.map(to_decimal)),
                ..Default::default()
            };
            am.insert(db).await?;
        }
        Ok(())
    }

    /// 批量 upsert
    pub async fn upsert_many(
        db: &DatabaseConnection,
        quotes: &[FundQuote],
    ) -> anyhow::Result<usize> {
        let mut count = 0;
        for q in quotes {
            Self::upsert(db, q).await?;
            count += 1;
        }
        Ok(count)
    }

    /// 分页查询
    pub async fn paginate(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        q: Option<&str>,
    ) -> anyhow::Result<(Vec<fund::Model>, u64)> {
        let mut query = fund::Entity::find();
        if let Some(q) = q {
            if !q.is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(fund::Column::Code.contains(q))
                        .add(fund::Column::Name.contains(q))
                        .add(fund::Column::Management.contains(q)),
                );
            }
        }
        let total = query.clone().count(db).await?;
        let rows = query
            .order_by_asc(fund::Column::Code)
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
    ) -> anyhow::Result<Option<fund::Model>> {
        fund::Entity::find()
            .filter(fund::Column::Code.eq(code))
            .one(db)
            .await
            .map_err(Into::into)
    }

    /// 获取全市场基金代码列表
    pub async fn list_all_codes(db: &DatabaseConnection) -> anyhow::Result<Vec<String>> {
        let rows = fund::Entity::find()
            .order_by_asc(fund::Column::Code)
            .all(db)
            .await?;
        Ok(rows.into_iter().map(|m| m.code).collect())
    }
}
