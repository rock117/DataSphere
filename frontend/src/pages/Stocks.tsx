import { useEffect, useState } from 'react'
import { Table, Input, Button, Space, message } from 'antd'
import { ReloadOutlined, SearchOutlined } from '@ant-design/icons'
import { api, Stock, Paginated } from '../api/client'

export default function Stocks() {
  const [data, setData] = useState<Paginated<Stock>>({ items: [], total: 0, page: 1, per_page: 20 })
  const [loading, setLoading] = useState(false)
  const [page, setPage] = useState(1)
  const [perPage] = useState(20)
  const [q, setQ] = useState('')

  const load = async () => {
    setLoading(true)
    try {
      const res = await api.listStocks({ page, per_page: perPage, q: q || undefined })
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
    { title: '符号', dataIndex: 'symbol', width: 120 },
    { title: '名称', dataIndex: 'name' },
    { title: '市场', dataIndex: 'market', width: 80 },
    { title: '交易所', dataIndex: 'exchange' },
    { title: '上市日期', dataIndex: 'list_date', width: 120, render: (v: string | null) => v || '-' },
  ]

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
