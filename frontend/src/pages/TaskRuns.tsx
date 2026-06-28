import { useEffect, useState, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Table, Button, Tag, Space, Progress, message, Typography } from "antd";
import { ArrowLeftOutlined, ReloadOutlined } from "@ant-design/icons";
import { api, Task, TaskRun } from "../api/client";

const { Title } = Typography;

const statusColor: Record<string, string> = {
  Running: "processing",
  Success: "success",
  Partial: "warning",
  Cancelled: "default",
  Failed: "error",
  Pending: "default",
};

function fmtDuration(ms: number): string {
  if (!ms) return "-";
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
  const m = Math.floor(ms / 60_000);
  const s = Math.round((ms % 60_000) / 1000);
  return `${m}m${s}s`;
}

export default function TaskRuns() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const taskId = Number(id);
  const [task, setTask] = useState<Task | null>(null);
  const [runs, setRuns] = useState<TaskRun[]>([]);
  const [loading, setLoading] = useState(false);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const loadTask = async () => {
    try {
      const t = await api.getTask(taskId);
      setTask(t || null);
    } catch (e: any) {
      message.error(e.message);
    }
  };

  const loadRuns = async () => {
    try {
      const r = await api.listRuns(taskId, 100);
      setRuns(r);
    } catch (e: any) {
      message.error(e.message);
    }
  };

  const loadAll = async () => {
    setLoading(true);
    await Promise.all([loadTask(), loadRuns()]);
    setLoading(false);
  };

  useEffect(() => {
    loadAll();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [taskId]);

  const hasRunning = runs.some((r) => r.status === "Running");

  useEffect(() => {
    if (hasRunning) {
      timerRef.current = setInterval(loadRuns, 2000);
    } else if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hasRunning]);

  const cancelRun = async (runId: number) => {
    try {
      const ok = await api.cancelRun(runId);
      if (ok) {
        message.success("已请求取消");
        loadRuns();
      } else {
        message.warning("无法取消（可能已结束）");
      }
    } catch (e: any) {
      message.error(e.message);
    }
  };

  const columns = [
    { title: "Run ID", dataIndex: "id", width: 70 },
    {
      title: "状态",
      dataIndex: "status",
      width: 100,
      render: (s: string) => <Tag color={statusColor[s]}>{s}</Tag>,
    },
    { title: "触发", dataIndex: "trigger_type", width: 80 },
    { title: "开始时间", dataIndex: "started_at", width: 170 },
    {
      title: "结束时间",
      dataIndex: "finished_at",
      width: 170,
      render: (v: string | null) => v || "-",
    },
    {
      title: "耗时",
      dataIndex: "duration_ms",
      width: 90,
      render: (v: number) => fmtDuration(v),
    },
    {
      title: "进度",
      width: 160,
      render: (_: unknown, r: TaskRun) => {
        if (r.status === "Running") {
          const pct = r.total > 0 ? Math.round((r.processed / r.total) * 100) : 0;
          return (
            <Progress
              percent={r.cancel_requested ? 100 : pct}
              size="small"
              status={r.cancel_requested ? "exception" : "active"}
              format={() => `${r.processed}/${r.total || "?"}`}
            />
          );
        }
        if (r.total > 0) return `${r.processed}/${r.total}`;
        return "-";
      },
    },
    {
      title: "成功",
      dataIndex: "success_count",
      width: 70,
      render: (v: number) => (v > 0 ? <span style={{ color: "#52c41a" }}>{v}</span> : v),
    },
    {
      title: "失败",
      dataIndex: "failed_count",
      width: 70,
      render: (v: number) => (v > 0 ? <span style={{ color: "#f5222d" }}>{v}</span> : v),
    },
    {
      title: "错误",
      dataIndex: "error",
      render: (v: string | null) => v || "-",
    },
    {
      title: "操作",
      width: 90,
      render: (_: unknown, r: TaskRun) =>
        r.status === "Running" && !r.cancel_requested ? (
          <Button size="small" danger onClick={() => cancelRun(r.id)}>
            取消
          </Button>
        ) : null,
    },
  ];

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Button icon={<ArrowLeftOutlined />} onClick={() => navigate("/tasks")}>
            返回任务列表
          </Button>
          <Button icon={<ReloadOutlined />} onClick={loadAll}>
            刷新
          </Button>
        </Space>
      </div>
      <Title level={4}>
        执行历史{task ? ` - #${task.id} ${task.name}` : ""}
      </Title>
      <Table
        rowKey="id"
        columns={columns}
        dataSource={runs}
        loading={loading}
        size="small"
        pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
        scroll={{ y: 600 }}
      />
    </div>
  );
}
