import { useEffect, useState } from 'react'
import { Table, Input, DatePicker, Button, Space, message } from 'antd'
import { ReloadOutlined, SearchOutlined } from '@ant-design/icons'
import dayjs from 'dayjs'
import { api, Kline } from '../api/client'

const { RangePicker } = DatePicker

export default function Klines() {
  const [code, setCode] = useState('600000')
  const [range, setRange] = useState<[dayjs.Dayjs, dayjs.Dayjs]>([
    dayjs().subtract(30, 'day'),
    dayjs(),
  ])
  const [data, setData] = useState<Kline[]>([])
  const [loading, setLoading] = useState(false)

  const load = async () => {
    if (!code) return
    setLoading(true)
    try {
      const params: { start?: string; end?: string } = {}
      if (range[0]) params.start = range[0].format('YYYY-MM-DD')
      if (range[1]) params.end = range[1].format('YYYY-MM-DD')
      const res = await api.getKlines(code, params)
      setData(res)
    } catch (e: any) {
      message.error(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  const columns = [
    { title: '日期', dataIndex: 'date', width: 110 },
    { title: '开盘', dataIndex: 'open', width: 90, render: (v: number) => v.toFixed(2) },
    { title: '收盘', dataIndex: 'close', width: 90, render: (v: number) => v.toFixed(2) },
    { title: '最高', dataIndex: 'high', width: 90, render: (v: number) => v.toFixed(2) },
    { title: '最低', dataIndex: 'low', width: 90, render: (v: number) => v.toFixed(2) },
    {
      title: '涨跌幅', dataIndex: 'pct_change', width: 90,
      render: (v: number | null) => v == null ? '-' : `${v > 0 ? '+' : ''}${v.toFixed(2)}%`,
    },
    { title: '成交量', dataIndex: 'volume', width: 120 },
    { title: '成交额', dataIndex: 'amount', width: 140, render: (v: number) => v.toLocaleString() },
    { title: '换手率', dataIndex: 'turnover', width: 90, render: (v: number | null) => v == null ? '-' : `${v.toFixed(2)}%` },
  ]

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Input
            placeholder="股票代码"
            value={code}
            onChange={(e) => setCode(e.target.value)}
            style={{ width: 140 }}
          />
          <RangePicker
            value={range}
            onChange={(vals) => {
              if (vals && vals[0] && vals[1]) setRange([vals[0], vals[1]])
            }}
          />
          <Button type="primary" icon={<SearchOutlined />} onClick={load}>查询</Button>
          <Button icon={<ReloadOutlined />} onClick={load}>刷新</Button>
        </Space>
      </div>
      <Table
        rowKey="id"
        columns={columns}
        dataSource={data}
        loading={loading}
        size="small"
        pagination={{ pageSize: 50, showTotal: (t) => `共 ${t} 条` }}
        scroll={{ y: 600 }}
      />
    </div>
  )
}
