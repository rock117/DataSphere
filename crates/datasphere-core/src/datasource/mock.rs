use crate::datasource::DataSource;
use crate::domain::{
    FetchFundListParams, FetchKlineParams, FetchKlineRequest, FetchStockListParams, FundQuote,
    FundType, KlineQuote, Market, StockQuote,
};
use crate::error::Result;
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use rand::Rng;

/// 内置 Mock 数据源，生成随机假数据用于开发与测试。
pub struct MockDataSource;

impl MockDataSource {
    pub fn new() -> Self {
        Self
    }

    fn mock_stock_list() -> Vec<StockQuote> {
        // 一些真实存在的 A股代码作为 mock 数据
        let raw: &[(Market, &str, &str)] = &[
            (Market::SH, "600000", "浦发银行"),
            (Market::SH, "600009", "上海机场"),
            (Market::SH, "600519", "贵州茅台"),
            (Market::SH, "600036", "招商银行"),
            (Market::SZ, "000001", "平安银行"),
            (Market::SZ, "000002", "万科A"),
            (Market::SZ, "000858", "五粮液"),
            (Market::SZ, "002594", "比亚迪"),
            (Market::SZ, "300750", "宁德时代"),
            (Market::BJ, "430047", "诺思兰德"),
        ];

        raw.iter()
            .map(|(m, code, name)| {
                let mut q = StockQuote::new(*code, *name, *m);
                // 随机上市日期
                let mut rng = rand::thread_rng();
                let year = rng.gen_range(2000..=2020);
                let month = rng.gen_range(1..=12);
                let day = rng.gen_range(1..=28);
                q.list_date = NaiveDate::from_ymd_opt(year, month, day);
                q
            })
            .collect()
    }

    /// 生成基金列表 mock 数据
    fn mock_fund_list() -> Vec<FundQuote> {
        let raw: &[(FundType, &str, &str, &str, &str, Option<f64>)] = &[
            (
                FundType::Stock,
                "000001",
                "华夏成长",
                "华夏基金",
                "中国银行",
                Some(12.5),
            ),
            (
                FundType::Mixed,
                "000002",
                "华夏大盘精选",
                "华夏基金",
                "中国银行",
                Some(45.8),
            ),
            (
                FundType::Bond,
                "000003",
                "华夏债券A",
                "华夏基金",
                "招商银行",
                Some(8.2),
            ),
            (
                FundType::Monetary,
                "000004",
                "华夏现金增利",
                "华夏基金",
                "建设银行",
                Some(320.5),
            ),
            (
                FundType::Index,
                "000300",
                "沪深300ETF",
                "华泰柏瑞",
                "招商银行",
                Some(1580.3),
            ),
            (
                FundType::Index,
                "510300",
                "沪深300ETF",
                "华泰柏瑞",
                "中国银行",
                Some(956.7),
            ),
            (
                FundType::Stock,
                "000005",
                "嘉实主题精选",
                "嘉实基金",
                "中国银行",
                Some(23.1),
            ),
            (
                FundType::Mixed,
                "000006",
                "嘉实策略增长",
                "嘉实基金",
                "工商银行",
                Some(67.4),
            ),
            (
                FundType::Qdii,
                "000007",
                "嘉实海外中国",
                "嘉实基金",
                "中国银行",
                Some(15.9),
            ),
            (
                FundType::Bond,
                "000008",
                "嘉实超短债",
                "嘉实基金",
                "招商银行",
                Some(5.6),
            ),
            (
                FundType::Fof,
                "000009",
                "华夏聚惠FOF",
                "华夏基金",
                "交通银行",
                Some(3.2),
            ),
            (
                FundType::Stock,
                "000010",
                "易方达价值成长",
                "易方达",
                "工商银行",
                Some(89.7),
            ),
            (
                FundType::Index,
                "000011",
                "易方达上证50",
                "易方达",
                "交通银行",
                Some(124.5),
            ),
            (
                FundType::Mixed,
                "000012",
                "易方达科翔",
                "易方达",
                "中国银行",
                Some(34.6),
            ),
            (
                FundType::Bond,
                "000013",
                "易方达稳健收益",
                "易方达",
                "招商银行",
                Some(18.9),
            ),
        ];

        let mut rng = rand::thread_rng();
        raw.iter()
            .map(|(ft, code, name, mgmt, cust, scale)| {
                let year = rng.gen_range(2005..=2020);
                let month = rng.gen_range(1..=12);
                let day = rng.gen_range(1..=28);
                FundQuote {
                    code: code.to_string(),
                    name: name.to_string(),
                    fund_type: *ft,
                    management: mgmt.to_string(),
                    custodian: cust.to_string(),
                    setup_date: NaiveDate::from_ymd_opt(year, month, day),
                    latest_scale: *scale,
                }
            })
            .collect()
    }

    /// 生成单只股票在日期范围内的随机 OHLCV 行情
    fn mock_kline(code: &str, start: NaiveDate, end: NaiveDate) -> Vec<KlineQuote> {
        let mut rng = rand::thread_rng();
        let mut out = Vec::new();
        let mut prev_close: f64 = rng.gen_range(5.0..50.0);

        let mut date = start;
        while date <= end {
            // 跳过周末
            if date.weekday().num_days_from_monday() < 5 {
                let change_pct: f64 = rng.gen_range(-0.098..0.098); // ±9.8%
                let open: f64 = prev_close;
                let close = (open * (1.0 + change_pct) * 100.0).round() / 100.0;
                let high = open.max(close) * (1.0 + rng.gen_range(0.0..0.02));
                let low = open.min(close) * (1.0 - rng.gen_range(0.0..0.02));
                let volume = rng.gen_range(100_000..10_000_000) as i64;
                let amount = (close * volume as f64 * 100.0).round(); // 成交额(元)
                let turnover = rng.gen_range(0.1..5.0); // 换手率%

                out.push(KlineQuote {
                    code: code.to_string(),
                    date,
                    open: round2(open),
                    close: round2(close),
                    high: round2(high),
                    low: round2(low),
                    volume,
                    amount,
                    turnover: Some(round2(turnover)),
                    pct_change: Some(round2(change_pct * 100.0)),
                });

                prev_close = close;
            }
            date += chrono::Duration::days(1);
        }

        out
    }
}

impl Default for MockDataSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataSource for MockDataSource {
    fn name(&self) -> &str {
        "mock"
    }

    async fn fetch_stock_list(&self, params: &FetchStockListParams) -> Result<Vec<StockQuote>> {
        let mut list = Self::mock_stock_list();
        if let Some(market) = params.market {
            list.retain(|q| q.market == market);
        }
        Ok(list)
    }

    async fn fetch_fund_list(&self, params: &FetchFundListParams) -> Result<Vec<FundQuote>> {
        let mut list = Self::mock_fund_list();
        if let Some(ft) = params.fund_type {
            list.retain(|q| q.fund_type == ft);
        }
        Ok(list)
    }

    async fn fetch_kline(&self, req: &FetchKlineRequest) -> Result<Vec<KlineQuote>> {
        Ok(Self::mock_kline(&req.code, req.start, req.end))
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

// 便捷：从 FetchKlineParams 构造一组请求（codes 为空时用默认股票池）
impl FetchKlineParams {
    pub fn to_requests(&self, default_codes: &[String]) -> Vec<FetchKlineRequest> {
        let today = chrono::Local::now().date_naive();
        let start = self.start.unwrap_or(today - chrono::Duration::days(30));
        let end = self.end.unwrap_or(today);

        let codes: Vec<String> = if self.codes.is_empty() {
            default_codes.to_vec()
        } else {
            self.codes.clone()
        };

        codes
            .into_iter()
            .map(|code| FetchKlineRequest { code, start, end })
            .collect()
    }
}
