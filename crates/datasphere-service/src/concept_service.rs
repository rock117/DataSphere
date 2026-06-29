use datasphere_core::domain::{Concept, StockConcept};
use datasphere_entity::{concept, stock_concept};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

pub struct ConceptService;

impl ConceptService {
    /// upsert 单个概念（按 name 唯一）
    pub async fn upsert_concept(db: &DatabaseConnection, c: &Concept) -> anyhow::Result<i64> {
        let existing = concept::Entity::find()
            .filter(concept::Column::Name.eq(&c.name))
            .one(db)
            .await?;
        if let Some(m) = existing {
            // 已存在：如需更新描述则更新，否则直接返回 id
            if c.description.is_some() && c.description != m.description {
                let id = m.id;
                let mut am: concept::ActiveModel = m.into();
                am.description = Set(c.description.clone());
                am.update(db).await?;
                return Ok(id);
            }
            return Ok(m.id);
        }
        let am = concept::ActiveModel {
            name: Set(c.name.clone()),
            description: Set(c.description.clone()),
            ..Default::default()
        };
        let model = am.insert(db).await?;
        Ok(model.id)
    }

    /// upsert 单条股票-概念关联（按 stock_code + concept_id 唯一）
    pub async fn upsert_stock_concept(
        db: &DatabaseConnection,
        stock_code: &str,
        concept_id: i64,
    ) -> anyhow::Result<()> {
        let existing = stock_concept::Entity::find()
            .filter(stock_concept::Column::StockCode.eq(stock_code))
            .filter(stock_concept::Column::ConceptId.eq(concept_id))
            .one(db)
            .await?;
        if existing.is_none() {
            let am = stock_concept::ActiveModel {
                stock_code: Set(stock_code.to_string()),
                concept_id: Set(concept_id),
                ..Default::default()
            };
            am.insert(db).await?;
        }
        Ok(())
    }

    /// 批量写入概念及成分股关联
    /// 先 upsert 所有概念（拿到 concept_id），再 upsert 关联
    pub async fn upsert_all(
        db: &DatabaseConnection,
        concepts: &[Concept],
        stock_concepts: &[StockConcept],
    ) -> anyhow::Result<usize> {
        // 构建概念名 -> id 的映射
        let mut name_to_id = std::collections::HashMap::new();
        for c in concepts {
            let id = Self::upsert_concept(db, c).await?;
            name_to_id.insert(c.name.clone(), id);
        }

        let mut count = 0;
        for sc in stock_concepts {
            if let Some(&concept_id) = name_to_id.get(&sc.concept_name) {
                Self::upsert_stock_concept(db, &sc.stock_code, concept_id).await?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// 列出所有概念
    pub async fn list_all(db: &DatabaseConnection) -> anyhow::Result<Vec<concept::Model>> {
        concept::Entity::find()
            .order_by_asc(concept::Column::Name)
            .all(db)
            .await
            .map_err(Into::into)
    }

    /// 查询某概念的成分股代码列表
    pub async fn list_stocks_by_concept(
        db: &DatabaseConnection,
        concept_id: i64,
    ) -> anyhow::Result<Vec<String>> {
        let rows = stock_concept::Entity::find()
            .filter(stock_concept::Column::ConceptId.eq(concept_id))
            .all(db)
            .await?;
        Ok(rows.into_iter().map(|m| m.stock_code).collect())
    }

    /// 查询某股票所属的所有概念
    pub async fn list_concepts_by_stock(
        db: &DatabaseConnection,
        stock_code: &str,
    ) -> anyhow::Result<Vec<concept::Model>> {
        let sc_rows = stock_concept::Entity::find()
            .filter(stock_concept::Column::StockCode.eq(stock_code))
            .all(db)
            .await?;
        let concept_ids: Vec<i64> = sc_rows.into_iter().map(|m| m.concept_id).collect();
        if concept_ids.is_empty() {
            return Ok(vec![]);
        }
        concept::Entity::find()
            .filter(concept::Column::Id.is_in(concept_ids))
            .all(db)
            .await
            .map_err(Into::into)
    }
}
