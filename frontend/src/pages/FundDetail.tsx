import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { Table, Button, Tag, Space, Select, Descriptions, message } from 'antd'
import { ArrowLeftOutlined, ReloadOutlined } from '@ant-design/icons'
import { api, Fund, FundHolding } from '../api/client'

const fundTypeColor: Record<string, string> = {
  '股票型': 'red',
  '混合型': 'orange',
  '债券型': 'blue',
  '货币型': 'green',
  '指数型': 'purple',
  'QDII': 'cyan',
  'FOF': 'magenta',
  '其他': 'default',
}

export default function FundDetail() {
  const { code } = useParams<{ code: string }>()
  const navigate = useNavigate()
  const [fund, setFund] = useState<Fund | null>(null)
  const [holdings, setHoldings] = useState<FundHolding[]>([])
  const [reportDates, setReportDates] = useState<string[]>([])
  const [selectedDate, setSelectedDate] = useState<string | undefined>(undefined)
  const [loading, setLoading] = useState(false)

  const loadFund = async () => {
    if (!code) return
    try {
      const f = await api.getFund(code)
      setFund(f || null)
    } catch (e: any) {
      message.error(e.message)
    }
  }

  const loadReportDates = async () => {
    if (!code) return
    try {
      const dates = await api.listReportDates(code)
      setReportDates(dates)
      // 默认选最新一期
      if (dates.length > 0 && !selectedDate) {
        setSelectedDate(dates[0])
      }
    } catch (e: any) {
      message.error(e.message)
    }
  }

  const loadHoldings = async () => {
    if (!code) return
    setLoading(true)
    try {
      let rows: FundHolding[]
      if (selectedDate) {
        rows = await api.listFundHoldingsByDate(code, selectedDate)
      } else {
        rows = await api.listFundHoldings(code, 50)
      }
      setHoldings(rows)
    } catch (e: any) {
      message.error(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadFund()
    loadReportDates()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [code])

  useEffect(() => {
    loadHoldings()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [code, selectedDate])

  const columns = [
    { title: '排名', dataIndex: 'rank', width: 60 },
    { title: '股票代码', dataIndex: 'stock_code', width: 100 },
    { title: '股票名称', dataIndex: 'stock_name' },
    {
      title: '占净值(%)', dataIndex: 'weight', width: 100,
      render: (v: number) => `${v.toFixed(2)}%`,
    },
    {
      title: '持仓股数', dataIndex: 'shares', width: 120,
      render: (v: number | null) => v == null ? '-' : v.toLocaleString(),
    },
    {
      title: '持仓市值(元)', dataIndex: 'market_value', width: 140,
      render: (v: number | null) => v == null ? '-' : v.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }),
    },
  ]

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Button icon={<ArrowLeftOutlined />} onClick={() => navigate('/funds')}>返回基金列表</Button>
          <Button icon={<ReloadOutlined />} onClick={() => { loadFund(); loadReportDates(); loadHoldings(); }}>刷新</Button>
        </Space>
      </div>

      {fund && (
        <Descriptions title={`基金详情 - ${fund.code} ${fund.name}`} bordered size="small" column={2} style={{ marginBottom: 16 }}>
          <Descriptions.Item label="代码">{fund.code}</Descriptions.Item>
          <Descriptions.Item label="名称">{fund.name}</Descriptions.Item>
          <Descriptions.Item label="类型">
            <Tag color={fundTypeColor[fund.fund_type] || 'default'}>{fund.fund_type}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="管理人">{fund.management}</Descriptions.Item>
          <Descriptions.Item label="托管人">{fund.custodian}</Descriptions.Item>
          <Descriptions.Item label="成立日期">{fund.setup_date || '-'}</Descriptions.Item>
          <Descriptions.Item label="最新规模(亿)">
            {fund.latest_scale == null ? '-' : fund.latest_scale.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </Descriptions.Item>
        </Descriptions>
      )}

      <div style={{ marginBottom: 16 }}>
        <Space>
          <span>报告期：</span>
          <Select
            style={{ width: 200 }}
            value={selectedDate}
            onChange={(v) => setSelectedDate(v)}
            options={reportDates.map((d) => ({ value: d, label: d }))}
            placeholder="选择报告期"
            allowClear
          />
        </Space>
      </div>

      <Table
        rowKey="id"
        columns={columns}
        dataSource={holdings}
        loading={loading}
        size="small"
        pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 条` }}
      />
    </div>
  )
}
