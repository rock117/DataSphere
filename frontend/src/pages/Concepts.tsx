import { useEffect, useState } from 'react'
import { Table, Button, Space, Tag, message, Modal } from 'antd'
import { ReloadOutlined, EyeOutlined } from '@ant-design/icons'
import { api, Concept, Stock } from '../api/client'

export default function Concepts() {
  const [concepts, setConcepts] = useState<Concept[]>([])
  const [loading, setLoading] = useState(false)
  const [modalOpen, setModalOpen] = useState(false)
  const [currentConcept, setCurrentConcept] = useState<Concept | null>(null)
  const [stocks, setStocks] = useState<Stock[]>([])
  const [stocksLoading, setStocksLoading] = useState(false)

  const load = async () => {
    setLoading(true)
    try {
      const list = await api.listConcepts()
      setConcepts(list)
    } catch (e: any) {
      message.error(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  const viewStocks = async (concept: Concept) => {
    setCurrentConcept(concept)
    setModalOpen(true)
    setStocksLoading(true)
    try {
      const list = await api.listConceptStocks(concept.id)
      setStocks(list)
    } catch (e: any) {
      message.error(e.message)
    } finally {
      setStocksLoading(false)
    }
  }

  const columns = [
    { title: 'ID', dataIndex: 'id', width: 60 },
    { title: '概念名称', dataIndex: 'name' },
    { title: '描述', dataIndex: 'description', render: (v: string | null) => v || '-' },
    {
      title: '操作', width: 110,
      render: (_: unknown, r: Concept) => (
        <Button size="small" icon={<EyeOutlined />} onClick={() => viewStocks(r)}>成分股</Button>
      ),
    },
  ]

  const stockColumns = [
    { title: '代码', dataIndex: 'code', width: 100 },
    { title: '名称', dataIndex: 'name' },
    { title: '市场', dataIndex: 'market', width: 80 },
    { title: '行业', dataIndex: 'industry', width: 120, render: (v: string | null) => v ? <Tag color="blue">{v}</Tag> : '-' },
  ]

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Button icon={<ReloadOutlined />} onClick={load}>刷新</Button>
        </Space>
      </div>
      <Table
        rowKey="id"
        columns={columns}
        dataSource={concepts}
        loading={loading}
        size="middle"
        pagination={{ pageSize: 20, showTotal: (t) => `共 ${t} 个概念` }}
      />

      <Modal
        title={currentConcept ? `成分股 - ${currentConcept.name}` : '成分股'}
        open={modalOpen}
        onCancel={() => setModalOpen(false)}
        footer={null}
        width={720}
      >
        <Table
          rowKey="id"
          size="small"
          columns={stockColumns}
          dataSource={stocks}
          loading={stocksLoading}
          pagination={{ pageSize: 20 }}
        />
      </Modal>
    </div>
  )
}
