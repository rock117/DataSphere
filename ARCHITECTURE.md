# DataSphere 架构设计：可扩展的数据采集框架

## 背景

随着数据类型（股票列表、日K、基金、行业、概念、龙虎榜...）和数据源（Tushare、AKShare、东财...）持续增长，需要一套可扩展的架构，使得：

- **加新数据类型**：只需加一个 Collector 文件 + 注册一行，不碰 runner / DataSource trait
- **加新数据源**：只需实现 DataSource trait + 声明 capabilities，不碰 Collector
- **数据源与数据类型解耦**：某些数据类型只有特定数据源支持，可灵活组合

## 核心问题与现状

### 现状痛点

| 痛点 | 说明 |
|------|------|
| runner 巨型 match | 6 个 TaskType 分支堆在一个函数，~400 行，每加类型要改 |
| DataSource trait 膨胀 | 每加类型加一个方法 + default 实现，trait 越来越胖 |
| 新增类型改动散落 | TaskType enum → DataSource trait → Mock → runner → entity → service → API → 前端 |

### 目标

- runner 稳定（~100 行），不因加数据类型而改动
- 加新数据类型 = 加 1 个 Collector 文件 + 注册 1 行
- 加新数据源 = 实现 DataSource + 声明能力，不碰 Collector
- 保留 enum 的编译期穷尽性检查

## 设计

### 总览

```
任务(task_type=StockList, provider=mock)
  │
  ▼
runner（调度核心，稳定不变）
  │  1. 查 CollectorRegistry 拿 StockListCollector
  │  2. 查 DataSourceRegistry 拿 MockDataSource（校验 capability）
  │  3. 创建 CollectContext
  │  4. collector.collect(ctx)
  ▼
StockListCollector（每个数据类型一个）
  │  - 调 source.fetch(DataType::StockList, params)
  │  - 调 StockService::upsert_many 落库
  │  - 累计 RunStats（进度/取消检查）
  ▼
DataSource（数据源抽象）
  │  - capabilities() 声明支持哪些 DataType
  │  - fetch(dt, params) 统一拉取入口
  ▼
DB（stocks/klines/funds/...）
```

### 核心抽象

#### DataType（enum）

数据类型标识，编译期穷尽性检查。

```rust
// datasphere-core/src/domain/data_type.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    StockList,
    Industry,
    Concept,
    FundList,
    FundHolding,
    Kline,
    // 新增数据类型在此扩展...
}
```

**为什么用 enum 而非字符串**：
- 编译期穷尽性检查——漏注册 Collector 时编译器报错
- 类型安全——不可能拼写错误
- IDE 自动补全、跳转
- 单体仓库场景下，所有类型都在 core 中定义，不需要第三方扩展

#### FetchParams / FetchResult（通用参数与结果）

替代每种类型一个 Params struct，用 JSON 封装：

```rust
// datasphere-core/src/domain/fetch.rs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FetchParams {
    pub data_type: DataType,
    /// 通用参数，各 Collector 按需解析
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchResult {
    Stocks(Vec<StockQuote>),
    Industries(Vec<StockIndustry>),
    Concepts(Vec<Concept>, Vec<StockConcept>),
    Funds(Vec<FundQuote>),
    FundHoldings(Vec<FundHolding>),
    Klines(Vec<KlineQuote>),
    // 新增类型加一个变体...
}
```

#### DataSource trait（数据源抽象，能力声明）

```rust
// datasphere-core/src/datasource/mod.rs
#[async_trait]
pub trait DataSource: Send + Sync + 'static {
    fn name(&self) -> &str;

    /// 声明支持的数据类型
    fn capabilities(&self) -> &[DataType];

    /// 统一拉取入口，按 data_type 分发
    async fn fetch(&self, dt: DataType, params: &FetchParams) -> Result<FetchResult>;
}
```

**能力矩阵示例**：

| | mock | tushare | akshare | eastmoney |
|---|---|---|---|---|
| StockList | ✅ | ✅ | ✅ | ✅ |
| Kline | ✅ | ✅ | ✅ | ❌ |
| DragonTiger | ❌ | ✅ | ❌ | ✅ |

#### Collector trait（采集器，每个数据类型一个）

```rust
// datasphere-service/src/collector/mod.rs
#[async_trait]
pub trait Collector: Send + Sync + 'static {
    /// 处理的数据类型
    fn data_type(&self) -> DataType;

    /// 执行采集
    async fn collect(&self, ctx: &CollectContext) -> anyhow::Result<RunStats>;
}

pub struct CollectContext {
    pub db: DatabaseConnection,
    pub source: Arc<dyn DataSource>,
    pub params: serde_json::Value,
    pub run_id: i64,
    pub update_progress: ProgressFn,
    pub is_cancelled: CancelFn,
}
```

**每个数据类型一个文件**：

| 文件 | Collector | 职责 |
|------|-----------|------|
| `collectors/stock_list.rs` | `StockListCollector` | 拉 stock list → stocks 表 |
| `collectors/industry.rs` | `IndustryCollector` | 拉 industry → 更新 stocks.industry |
| `collectors/concept.rs` | `ConceptCollector` | 拉 concepts → concepts + stock_concepts |
| `collectors/fund_list.rs` | `FundListCollector` | 拉 fund list → funds 表 |
| `collectors/fund_holding.rs` | `FundHoldingCollector` | 拉 fund holdings → fund_holdings |
| `collectors/kline.rs` | `KlineCollector` | 拉 kline → klines 表 |

#### CollectorRegistry（采集器注册表）

```rust
pub struct CollectorRegistry {
    collectors: HashMap<DataType, Arc<dyn Collector>>,
}

impl CollectorRegistry {
    pub fn register(&mut self, collector: Arc<dyn Collector>);
    pub fn get(&self, dt: &DataType) -> Option<Arc<dyn Collector>>;
}
```

#### runner 简化

```rust
async fn execute_inner(&self, task_model, run_id) -> Result<RunStats> {
    let dt: DataType = task_model.task_type.parse()?;
    let collector = self.collectors.get(&dt)
        .ok_or_else(|| anyhow::anyhow!("no collector for {dt}"))?;
    let source = self.registry.get(&task_model.provider)?;

    // 校验数据源是否支持该数据类型
    if !source.capabilities().contains(&dt) {
        anyhow::bail!("data source '{}' does not support {dt}", source.name());
    }

    let ctx = CollectContext { db, source, params, run_id, ... };
    collector.collect(&ctx).await
}
```

**runner 不再有 match 分支**，只做调度（锁/进度/取消/统计）。

### 扩展场景

#### 加新数据类型（如龙虎榜）

1. `DataType` enum 加 `DragonTiger` 变体
2. `FetchResult` 加 `DragonTiger(Vec<DragonTigerRecord>)` 变体
3. 新建 `collectors/dragon_tiger.rs` 实现 `Collector`
4. `CollectorRegistry` 注册一行
5. 数据源按需在 `capabilities()` 声明 + `fetch()` 分发

**不碰 runner、不碰其他 Collector**。

#### 加新数据源（如 Tushare）

1. 新建 `datasphere-core/src/datasource/tushare.rs` 实现 `DataSource`
2. `capabilities()` 声明支持的数据类型
3. `fetch()` 按 DataType 分发到具体 API 调用
4. `main.rs` 注册一行

**不碰 Collector、不碰 runner**。

#### 数据源独有数据类型

数据源 A 独有龙虎榜：
- A 的 `capabilities()` 包含 `DragonTiger`
- `DragonTigerCollector` 存在并注册
- 创建任务时 `provider=A, task_type=DragonTiger` 可用
- 创建任务时 `provider=B, task_type=DragonTiger` → runner 校验 capability 失败，返回清晰错误

## API 增强

```
GET /api/datasources                          # 列出所有数据源
GET /api/datasources?capability=stock_list    # 列出支持某数据类型的数据源
GET /api/data_types                           # 列出所有数据类型
GET /api/data_types/:dt/collectors            # 列出已注册的 Collector
```

前端创建任务时：
1. 选数据类型 → 查询支持的数据源 → 选数据源
2. 避免选了不支持的数据源组合

## 文件结构

```
crates/
├── datasphere-core/
│   └── src/
│       ├── domain/
│       │   ├── data_type.rs      # DataType enum
│       │   ├── fetch.rs          # FetchParams / FetchResult
│       │   └── ...               # 现有 domain
│       └── datasource/
│           ├── mod.rs            # DataSource trait
│           ├── registry.rs       # DataSourceRegistry
│           └── mock.rs           # MockDataSource
├── datasphere-service/
│   └── src/
│       ├── collector/
│       │   ├── mod.rs            # Collector trait + CollectorRegistry + CollectContext
│       │   ├── stock_list.rs
│       │   ├── industry.rs
│       │   ├── concept.rs
│       │   ├── fund_list.rs
│       │   ├── fund_holding.rs
│       │   └── kline.rs
│       ├── runner.rs             # 简化后的 runner（~100 行）
│       └── ...                   # 现有 service
```

## 收益对比

| 方面 | 重构前 | 重构后 |
|------|--------|--------|
| 加新数据类型碰 runner | ✅ 加 match 分支 | ❌ 不碰 |
| runner 行数 | ~400 行且增长 | ~100 行稳定 |
| 采集逻辑位置 | 散在 runner match 里 | 每个类型独立文件 |
| 可测试性 | 难单独测 | Collector 可独立测 |
| 数据源能力查询 | 不支持 | capability 声明 |
| 编译期检查 | TaskType enum | DataType enum + Collector 注册 |

## 迁移计划

1. core: 新增 `DataType` / `FetchParams` / `FetchResult`
2. core: 重构 `DataSource` trait（capabilities + 统一 fetch）
3. core: 重构 `MockDataSource`（实现新 trait）
4. service: 新增 `Collector` trait + `CollectorRegistry` + `CollectContext`
5. service: 6 个数据类型各拆出 Collector
6. service: 简化 runner
7. api: 加 capability 查询端点
8. 验证: cargo check + tsc
9. entity/service/前端: 不变
