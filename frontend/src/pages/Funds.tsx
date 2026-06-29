import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Table, Input, Button, Space, Tag, message } from 'antd'
import { ReloadOutlined, SearchOutlined, EyeOutlined } from '@ant-design/icons'
import { api, Fund, Paginated } from '../api/client'

const fundTypeColor: Record<string, string> = {
  '股票型': 'red',
  '混合型': 'orange',
  '债券型': 'blue',
  '货币型': 'green',
  '指数型': 'purple',
  'ETF': 'geekblue',
  'QDII': 'cyan',
  'FOF': 'magenta',
  '其他': 'default',
}

export default function Funds() {
  const [data, setData] = useState<Paginated<Fund>>({ items: [], total: 0, page: 1, per_page: 20 })
  const [loading, setLoading] = useState(false)
  const [page, setPage] = useState(1)
  const [perPage] = useState(20)
  const [q, setQ] = useState('')
  const navigate = useNavigate()

  const load = async () => {
    setLoading(true)
    try {
      const res = await api.listFunds({ page, per_page: perPage, q: q || undefined })
      setData(res)
    } catch (e: any) {
      message.error(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [page])

  const columns = [
    { title: '代码', dataIndex: 'code', width: 100 },
    { title: '名称', dataIndex: 'name' },
    {
      title: '类型', dataIndex: 'fund_type', width: 90,
      render: (v: string) => <Tag color={fundTypeColor[v] || 'default'}>{v}</Tag>,
    },
    { title: '管理人', dataIndex: 'management', width: 140 },
    { title: '托管人', dataIndex: 'custodian', width: 140 },
    {
      title: '成立日期', dataIndex: 'setup_date', width: 120,
      render: (v: string | null) => v || '-',
    },
    {
      title: '最新规模(亿)', dataIndex: 'latest_scale', width: 120,
      render: (v: number | null) => v == null ? '-' : v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }),
    },
    {
      title: '操作', width: 110,
      render: (_: unknown, r: Fund) => (
        <Button size="small" icon={<EyeOutlined />} onClick={() => navigate(`/funds/${r.code}`)}>成分股</Button>
      ),
    },
  ]

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Input.Search
            placeholder="搜索代码/名称/管理人"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            onSearch={load}
            style={{ width: 240 }}
            enterButton={<SearchOutlined />}
          />
          <Button icon={<ReloadOutlined />} onClick={load}>刷新</Button>
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
  )
}
