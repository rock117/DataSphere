import { useEffect, useState } from "react";
import { Table, Input, Button, Space, Select, Tag, message } from "antd";
import { ReloadOutlined, SearchOutlined } from "@ant-design/icons";
import { api, Stock, Paginated } from "../api/client";

export default function Stocks() {
  const [data, setData] = useState<Paginated<Stock>>({
    items: [],
    total: 0,
    page: 1,
    per_page: 20,
  });
  const [loading, setLoading] = useState(false);
  const [page, setPage] = useState(1);
  const [perPage] = useState(20);
  const [q, setQ] = useState("");
  const [industry, setIndustry] = useState<string | undefined>(undefined);
  const [industries, setIndustries] = useState<string[]>([]);

  const load = async () => {
    setLoading(true);
    try {
      const res = await api.listStocks({
        page,
        per_page: perPage,
        q: q || undefined,
        industry,
      });
      setData(res);
    } catch (e: any) {
      message.error(e.message);
    } finally {
      setLoading(false);
    }
  };

  const loadIndustries = async () => {
    try {
      const list = await api.listIndustries();
      setIndustries(list);
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    load();
  }, [page]);
  useEffect(() => {
    loadIndustries();
  }, []);

  const columns = [
    { title: "代码", dataIndex: "code", width: 100 },
    { title: "符号", dataIndex: "symbol", width: 120 },
    { title: "名称", dataIndex: "name" },
    { title: "市场", dataIndex: "market", width: 80 },
    {
      title: "行业",
      dataIndex: "industry",
      width: 120,
      render: (v: string | null) => (v ? <Tag color="blue">{v}</Tag> : "-"),
    },
    { title: "交易所", dataIndex: "exchange" },
    {
      title: "上市日期",
      dataIndex: "list_date",
      width: 120,
      render: (v: string | null) => v || "-",
    },
  ];

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Input.Search
            placeholder="搜索代码或名称"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            onSearch={load}
            style={{ width: 240 }}
            enterButton={<SearchOutlined />}
          />
          <Select
            allowClear
            placeholder="按行业筛选"
            value={industry}
            onChange={(v) => {
              setIndustry(v);
              setPage(1);
            }}
            options={industries.map((i) => ({ value: i, label: i }))}
            style={{ width: 160 }}
          />
          <Button icon={<ReloadOutlined />} onClick={load}>
            刷新
          </Button>
        </Space>
      </div>
      <Table
        rowKey="id"
        columns={columns}
        dataSource={data.items}
        loading={loading}
        size="middle"
        pagination={{
          current: page,
          pageSize: perPage,
          total: data.total,
          onChange: (p) => setPage(p),
          showTotal: (t) => `共 ${t} 条`,
        }}
      />
    </div>
  );
}
