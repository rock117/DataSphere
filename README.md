# DataSphere — 金融数据收集系统

定时收集 + 手动触发的金融数据收集应用。后端 Rust (Rocket + SeaORM + MySQL)，前端 React + Ant Design。

当前支持 A股股票列表与日K历史行情，通过 `DataSource` 抽象层实现数据源可扩展、可切换（内置 Mock 数据源）。

## 技术栈

- **后端**: Rust + Rocket 0.5 + SeaORM 2.0 + MySQL
- **调度**: tokio-cron-scheduler（自定义 cron 表达式）
- **前端**: React 18 + TypeScript + Vite + Ant Design 5

## 项目结构

```
DataSphere/
├── Cargo.toml                  # workspace 根
├── config.toml                 # 配置文件
├── migrations/
│   └── 001_init.sql            # 建表脚本
├── crates/
│   ├── datasphere-core/        # 核心抽象：DataSource trait、领域 DTO、Mock 数据源
│   ├── datasphere-entity/      # SeaORM entity（stock/kline/task/task_run）
│   ├── datasphere-service/     # 业务服务 + 任务执行器（upsert、CRUD、runner）
│   ├── datasphere-scheduler/   # tokio-cron-scheduler 封装（动态增删启停）
│   └── datasphere-api/         # Rocket 主二进制（REST API）
└── frontend/                   # React 前端
```

## 数据源抽象

核心 trait `DataSource` 定义在 `datasphere-core` 中：

```rust
#[async_trait]
pub trait DataSource: Send + Sync + 'static {
    fn name(&self) -> &str;
    async fn fetch_stock_list(&self, params: &FetchStockListParams) -> Result<Vec<StockQuote>>;
    async fn fetch_kline(&self, req: &FetchKlineRequest) -> Result<Vec<KlineQuote>>;
}
```

**扩展新数据源**：实现 `DataSource` trait → 在 `DataSourceRegistry` 中注册 → 任务通过 `provider` 字段选择。

内置 `MockDataSource`（name="mock"）生成随机 A股代码与 OHLCV 行情，用于开发测试。

## 快速开始

### 前置要求

- Rust 1.94+（`rustup update stable`）
- MySQL 8.0+
- Node.js 18+（前端）

### 1. 初始化数据库

使用 sqlx-cli 执行迁移（自动创建表 + 追踪已执行版本）：

```bash
# 安装 sqlx-cli（如未安装）
cargo install sqlx-cli --no-default-features --features mysql,rustls

# 执行迁移（DATABASE_URL 可放在 .env 中，sqlx 会自动读取）
DATABASE_URL=mysql://root:123456@localhost:3306/datasphere sqlx migrate run
```

后续新增迁移文件后，再次执行 `sqlx migrate run` 会自动跑未执行的版本。

新建空迁移文件：
```bash
sqlx migrate add -r <name>   # -r 生成可回滚的 up/down 文件
```

### 2. 配置

配置分两层，职责清晰、无冗余：

- `config.toml`：业务配置（端口、连接池大小、数据源、时区）—— 无敏感信息，可提交 git
- `.env`：敏感信息（`DATABASE_URL` 含密码、`RUST_LOG`）—— 由 `dotenvy` 自动加载，gitignore 不提交

```bash
cp .env.example .env
# 编辑 .env 中的 DATABASE_URL 为你的 MySQL 连接串（含密码）
# config.toml 一般无需改动
```

> 数据库连接串只在 `.env` 的 `DATABASE_URL` 中配置一处，`config.toml` 不再包含任何连接串，避免敏感信息进版本库。

### 3. 启动后端

```bash
cargo run -p datasphere-api
```

后端默认监听 `http://127.0.0.1:8000`。

### 4. 启动前端

```bash
cd frontend
npm install
npm run dev
```

前端默认监听 `http://localhost:5173`，自动代理 `/api` 到后端。

## 使用

### 通过前端

1. **任务管理页**：创建任务、手动触发、重新拉取、查看执行历史
2. **股票列表页**：分页查询、搜索
3. **行情查看页**：按代码与日期范围查询日K

### 通过 API

```bash
# 健康检查
curl http://127.0.0.1:8000/api/health

# 创建拉取股票列表任务（Mock）
curl -X POST http://127.0.0.1:8000/api/tasks \
  -H 'Content-Type: application/json' \
  -d '{"name":"拉取A股列表","task_type":"FetchStockList","provider":"mock","enabled":true}'

# 手动触发任务
curl -X POST http://127.0.0.1:8000/api/tasks/1/run

# 创建定时拉取日K任务（工作日18点执行）
curl -X POST http://127.0.0.1:8000/api/tasks \
  -H 'Content-Type: application/json' \
  -d '{"name":"每日行情","task_type":"FetchKline","provider":"mock","cron":"0 0 18 * * 1-5","params":{"start":"2024-01-01","end":"2024-06-28"},"enabled":true}'

# 查询股票
curl 'http://127.0.0.1:8000/api/stocks?page=1&per_page=20&q=浦发'

# 查询行情
curl 'http://127.0.0.1:8000/api/klines/600000?start=2024-01-01&end=2024-06-01'

# 查看执行历史
curl http://127.0.0.1:8000/api/tasks/1/runs
```

## API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/health` | 健康检查 |
| GET | `/api/datasources` | 列出已注册数据源 |
| GET | `/api/stocks?page&per_page&q` | 分页查询股票 |
| GET | `/api/stocks/:code` | 查单只股票 |
| GET | `/api/funds?page&per_page&q` | 分页查询基金 |
| GET | `/api/funds/:code` | 查单只基金 |
| GET | `/api/funds/:code/holdings?limit` | 基金成分股（最新报告期） |
| GET | `/api/funds/:code/holdings/:report_date` | 基金成分股（指定报告期） |
| GET | `/api/funds/:code/report_dates` | 基金所有报告期 |
| GET | `/api/klines/:code?start&end` | 查询日K行情 |
| GET | `/api/tasks` | 列出所有任务（含最近运行摘要） |
| GET | `/api/tasks/:id` | 查单条任务 |
| POST | `/api/tasks` | 创建任务 |
| PUT | `/api/tasks/:id` | 更新任务 |
| POST | `/api/tasks/:id/toggle` | 一键启停 |
| DELETE | `/api/tasks/:id` | 删除任务 |
| POST | `/api/tasks/:id/run` | 手动触发 |
| POST | `/api/tasks/:id/refetch` | 重新拉取 |
| GET | `/api/tasks/:id/next_run?count` | 预览下次执行时间 |
| GET | `/api/tasks/:id/runs?limit` | 执行历史 |
| GET | `/api/runs/:run_id` | 查单条运行记录 |
| POST | `/api/runs/:run_id/cancel` | 请求取消运行 |

## 扩展数据源示例

以接入 Tushare 为例（伪代码）：

```rust
// 1. 实现 DataSource trait
pub struct TushareDataSource { token: String }

#[async_trait]
impl DataSource for TushareDataSource {
    fn name(&self) -> &str { "tushare" }
    async fn fetch_stock_list(&self, params: &FetchStockListParams) -> Result<Vec<StockQuote>> {
        // 调用 Tushare API ...
    }
    async fn fetch_kline(&self, req: &FetchKlineRequest) -> Result<Vec<KlineQuote>> {
        // 调用 Tushare API ...
    }
}

// 2. 在 main.rs 中注册
registry.register(Arc::new(TushareDataSource::new(token)));

// 3. 创建任务时指定 provider="tushare"
```
