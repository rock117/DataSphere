import axios from 'axios';

const client = axios.create({
  baseURL: '/api',
  timeout: 30000,
});

// 统一解析 ApiResponse 格式 { ok: { data } } / { err: { error } }
client.interceptors.response.use(
  (resp) => {
    const body = resp.data;
    if (body && body.ok) return body.ok.data;
    if (body && body.err) return Promise.reject(new Error(body.err.error));
    return body;
  },
  (error) => {
    const msg = error.response?.data?.err?.error || error.message;
    return Promise.reject(new Error(msg));
  }
);

export default client;

// ---- 类型 ----

export interface Stock {
  id: number;
  code: string;
  symbol: string;
  name: string;
  market: string;
  exchange: string;
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
}

export interface TaskRun {
  id: number;
  task_id: number;
  status: string;
  trigger_type: string;
  started_at: string;
  finished_at: string | null;
  records_affected: number;
  error: string | null;
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
  health: () => client.get<string>('/health'),

  // stocks
  listStocks: (params: { page?: number; per_page?: number; q?: string }) =>
    client.get<Paginated<Stock>>('/stocks', { params }),
  getStock: (code: string) => client.get<Stock | null>(`/stocks/${code}`),

  // klines
  getKlines: (code: string, params: { start?: string; end?: string }) =>
    client.get<Kline[]>(`/klines/${code}`, { params }),

  // tasks
  listTasks: () => client.get<Task[]>('/tasks'),
  getTask: (id: number) => client.get<Task | null>(`/tasks/${id}`),
  createTask: (data: Partial<Task>) => client.post<Task>('/tasks', data),
  updateTask: (id: number, data: Partial<Task>) => client.put<Task | null>(`/tasks/${id}`, data),
  deleteTask: (id: number) => client.delete<boolean>(`/tasks/${id}`),
  runTask: (id: number) => client.post<number>(`/tasks/${id}/run`),
  refetchTask: (id: number) => client.post<number>(`/tasks/${id}/refetch`),

  // runs
  listRuns: (taskId: number, limit?: number) =>
    client.get<TaskRun[]>(`/tasks/${taskId}/runs`, { params: { limit } }),
  getRun: (runId: number) => client.get<TaskRun | null>(`/runs/${runId}`),
};
