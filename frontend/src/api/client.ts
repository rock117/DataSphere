import axios, { AxiosRequestConfig } from "axios";

const instance = axios.create({
  baseURL: "/api",
  timeout: 30000,
});

// 统一解析 ApiResponse 格式 { ok: { data } } / { err: { error } }
// 拦截器把 resp 解包成纯 data 返回，因此 request 包装层返回 Promise<T> 而非 AxiosResponse<T>
instance.interceptors.response.use(
  (resp) => {
    const body = resp.data;
    if (body && body.ok) return body.ok.data;
    if (body && body.err) return Promise.reject(new Error(body.err.error));
    return body;
  },
  (error) => {
    const msg = error.response?.data?.err?.error || error.message;
    return Promise.reject(new Error(msg));
  },
);

// 类型安全的请求封装：拦截器已解包，直接返回 T
async function request<T>(config: AxiosRequestConfig): Promise<T> {
  return (await instance.request(config)) as unknown as Promise<T>;
}

export default instance;
export { request };

// ---- 类型 ----

export interface Stock {
  id: number;
  code: string;
  symbol: string;
  name: string;
  market: string;
  exchange: string;
  industry: string | null;
  list_date: string | null;
  delist_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface Kline {
  id: number;
  code: string;
  date: string;
  open: number;
  close: number;
  high: number;
  low: number;
  volume: number;
  amount: number;
  turnover: number | null;
  pct_change: number | null;
}

export interface Task {
  id: number;
  name: string;
  task_type: string;
  provider: string;
  cron: string | null;
  params: Record<string, unknown> | null;
  enabled: boolean;
  created_at: string;
  updated_at: string;
  // list_with_last_run 附带的最近运行摘要
  last_run_id?: number | null;
  last_run_status?: string | null;
  last_run_at?: string | null;
  last_run_success?: number | null;
  last_run_failed?: number | null;
}

export interface TaskRun {
  id: number;
  task_id: number;
  status: string;
  trigger_type: string;
  started_at: string;
  finished_at: string | null;
  records_affected: number;
  success_count: number;
  failed_count: number;
  duration_ms: number;
  total: number;
  processed: number;
  cancel_requested: boolean;
  error: string | null;
}

export interface Fund {
  id: number;
  code: string;
  name: string;
  fund_type: string;
  management: string;
  custodian: string;
  setup_date: string | null;
  latest_scale: number | null;
  created_at: string;
  updated_at: string;
}

export interface FundHolding {
  id: number;
  fund_code: string;
  stock_code: string;
  stock_name: string;
  report_date: string;
  weight: number;
  shares: number | null;
  market_value: number | null;
  rank: number | null;
  created_at: string;
  updated_at: string;
}

export interface Concept {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface Paginated<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

// ---- API 调用 ----

export const api = {
  // health
  health: () => request<string>({ url: "/health" }),

  // stocks
  listStocks: (params: {
    page?: number;
    per_page?: number;
    q?: string;
    industry?: string;
  }) => request<Paginated<Stock>>({ url: "/stocks", params }),
  getStock: (code: string) => request<Stock | null>({ url: `/stocks/${code}` }),
  listIndustries: () => request<string[]>({ url: "/industries" }),

  // concepts
  listConcepts: () => request<Concept[]>({ url: "/concepts" }),
  listConceptStocks: (id: number) =>
    request<Stock[]>({ url: `/concepts/${id}/stocks` }),
  listStockConcepts: (code: string) =>
    request<Concept[]>({ url: `/stocks/${code}/concepts` }),

  // funds
  listFunds: (params: { page?: number; per_page?: number; q?: string }) =>
    request<Paginated<Fund>>({ url: "/funds", params }),
  getFund: (code: string) => request<Fund | null>({ url: `/funds/${code}` }),
  listFundHoldings: (code: string, limit?: number) =>
    request<FundHolding[]>({
      url: `/funds/${code}/holdings`,
      params: { limit },
    }),
  listFundHoldingsByDate: (code: string, reportDate: string) =>
    request<FundHolding[]>({ url: `/funds/${code}/holdings/${reportDate}` }),
  listReportDates: (code: string) =>
    request<string[]>({ url: `/funds/${code}/report_dates` }),

  // klines
  getKlines: (code: string, params: { start?: string; end?: string }) =>
    request<Kline[]>({ url: `/klines/${code}`, params }),

  // tasks
  listTasks: () => request<Task[]>({ url: "/tasks" }),
  getTask: (id: number) => request<Task | null>({ url: `/tasks/${id}` }),
  createTask: (data: Partial<Task>) =>
    request<Task>({ url: "/tasks", method: "POST", data }),
  updateTask: (id: number, data: Partial<Task>) =>
    request<Task | null>({ url: `/tasks/${id}`, method: "PUT", data }),
  deleteTask: (id: number) =>
    request<boolean>({ url: `/tasks/${id}`, method: "DELETE" }),
  toggleTask: (id: number) =>
    request<Task | null>({ url: `/tasks/${id}/toggle`, method: "POST" }),
  runTask: (id: number) =>
    request<number>({ url: `/tasks/${id}/run`, method: "POST" }),
  refetchTask: (id: number) =>
    request<number>({ url: `/tasks/${id}/refetch`, method: "POST" }),
  nextRun: (id: number, count?: number) =>
    request<string[]>({ url: `/tasks/${id}/next_run`, params: { count } }),

  // runs
  listRuns: (taskId: number, limit?: number) =>
    request<TaskRun[]>({ url: `/tasks/${taskId}/runs`, params: { limit } }),
  getRun: (runId: number) => request<TaskRun | null>({ url: `/runs/${runId}` }),
  cancelRun: (runId: number) =>
    request<boolean>({ url: `/runs/${runId}/cancel`, method: "POST" }),
};
