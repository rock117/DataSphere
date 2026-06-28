import { useEffect, useState } from 'react'
import { Table, Button, Modal, Form, Input, Select, Switch, Space, Tag, message, Drawer, Descriptions } from 'antd'
import { PlusOutlined, ReloadOutlined, ThunderboltOutlined, RedoOutlined, EyeOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons'
import { api, Task, TaskRun } from '../api/client'

const { TextArea } = Input

const statusColor: Record<string, string> = {
  Running: 'processing',
  Success: 'success',
  Failed: 'error',
  Pending: 'default',
}

export default function Tasks() {
  const [tasks, setTasks] = useState<Task[]>([])
  const [loading, setLoading] = useState(false)
  const [modalOpen, setModalOpen] = useState(false)
  const [editTask, setEditTask] = useState<Task | null>(null)
  const [form] = Form.useForm()
  const [runsDrawer, setRunsDrawer] = useState<{ open: boolean; task: Task | null; runs: TaskRun[] }>({
    open: false,
    task: null,
    runs: [],
  })

  const load = async () => {
    setLoading(true)
    try {
      const data = await api.listTasks()
      setTasks(data)
    } catch (e: any) {
      message.error(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  const openCreate = () => {
    setEditTask(null)
    form.resetFields()
    form.setFieldsValue({ provider: 'mock', enabled: true, task_type: 'FetchStockList' })
    setModalOpen(true)
  }

  const openEdit = (task: Task) => {
    setEditTask(task)
    form.setFieldsValue({
      ...task,
      params: task.params ? JSON.stringify(task.params, null, 2) : '',
    })
    setModalOpen(true)
  }

  const submit = async () => {
    try {
      const vals = await form.validateFields()
      let params: Record<string, unknown> | null = null
      if (vals.params) {
        try {
          params = JSON.parse(vals.params)
        } catch {
          message.error('params 不是合法 JSON')
          return
        }
      }
      const payload = { ...vals, params }
      if (editTask) {
        await api.updateTask(editTask.id, payload)
        message.success('更新成功')
      } else {
        await api.createTask(payload)
        message.success('创建成功')
      }
      setModalOpen(false)
      load()
    } catch (e: any) {
      if (e.errorFields) return
      message.error(e.message)
    }
  }

  const runTask = async (id: number) => {
    try {
      const runId = await api.runTask(id)
      message.success(`已触发，run_id=${runId}`)
      load()
    } catch (e: any) {
      message.error(e.message)
    }
  }

  const refetchTask = async (id: number) => {
    try {
      const runId = await api.refetchTask(id)
      message.success(`已重新拉取，run_id=${runId}`)
      load()
    } catch (e: any) {
      message.error(e.message)
    }
  }

  const deleteTask = async (id: number) => {
    Modal.confirm({
      title: '确认删除',
      content: `删除任务 #${id}？`,
      onOk: async () => {
        try {
          await api.deleteTask(id)
          message.success('已删除')
          load()
        } catch (e: any) {
          message.error(e.message)
        }
      },
    })
  }

  const viewRuns = async (task: Task) => {
    try {
      const runs = await api.listRuns(task.id, 50)
      setRunsDrawer({ open: true, task, runs })
    } catch (e: any) {
      message.error(e.message)
    }
  }

  const columns = [
    { title: 'ID', dataIndex: 'id', width: 60 },
    { title: '名称', dataIndex: 'name' },
    { title: '类型', dataIndex: 'task_type', width: 140 },
    { title: '数据源', dataIndex: 'provider', width: 100 },
    {
      title: 'Cron', dataIndex: 'cron', width: 160,
      render: (v: string | null) => v || <Tag>手动</Tag>,
    },
    {
      title: '启用', dataIndex: 'enabled', width: 80,
      render: (v: boolean) => <Tag color={v ? 'success' : 'default'}>{v ? '是' : '否'}</Tag>,
    },
    {
      title: '操作', width: 280, render: (_: unknown, r: Task) => (
        <Space size="small">
          <Button size="small" type="primary" icon={<ThunderboltOutlined />} onClick={() => runTask(r.id)}>触发</Button>
          <Button size="small" icon={<RedoOutlined />} onClick={() => refetchTask(r.id)}>重拉</Button>
          <Button size="small" icon={<EyeOutlined />} onClick={() => viewRuns(r)}>历史</Button>
          <Button size="small" icon={<EditOutlined />} onClick={() => openEdit(r)}>编辑</Button>
          <Button size="small" danger icon={<DeleteOutlined />} onClick={() => deleteTask(r.id)} />
        </Space>
      ),
    },
  ]

  return (
    <div>
      <div style={{ marginBottom: 16 }}>
        <Space>
          <Button type="primary" icon={<PlusOutlined />} onClick={openCreate}>新建任务</Button>
          <Button icon={<ReloadOutlined />} onClick={load}>刷新</Button>
        </Space>
      </div>
      <Table rowKey="id" columns={columns} dataSource={tasks} loading={loading} pagination={false} size="middle" />

      <Modal title={editTask ? '编辑任务' : '新建任务'} open={modalOpen} onOk={submit} onCancel={() => setModalOpen(false)} width={640} destroyOnClose>
        <Form form={form} layout="vertical">
          <Form.Item name="name" label="任务名称" rules={[{ required: true }]}>
            <Input />
          </Form.Item>
          <Form.Item name="task_type" label="任务类型" rules={[{ required: true }]}>
            <Select options={[{ value: 'FetchStockList', label: '拉取股票列表' }, { value: 'FetchKline', label: '拉取日K行情' }]} />
          </Form.Item>
          <Form.Item name="provider" label="数据源" rules={[{ required: true }]}>
            <Select options={[{ value: 'mock', label: 'Mock (假数据)' }]} />
          </Form.Item>
          <Form.Item name="cron" label="Cron 表达式" tooltip="留空则仅手动触发。例：0 0 18 * * 1-5（工作日18点）">
            <Input placeholder="如 0 0 18 * * 1-5" />
          </Form.Item>
          <Form.Item name="params" label="参数 (JSON)" tooltip='FetchKline 例：{"start":"2024-01-01","end":"2024-06-01","codes":["600000"]}'>
            <TextArea rows={4} placeholder='{}' />
          </Form.Item>
          <Form.Item name="enabled" label="启用" valuePropName="checked">
            <Switch />
          </Form.Item>
        </Form>
      </Modal>

      <Drawer
        title={runsDrawer.task ? `执行历史 - #${runsDrawer.task.id} ${runsDrawer.task.name}` : '执行历史'}
        open={runsDrawer.open}
        onClose={() => setRunsDrawer({ open: false, task: null, runs: [] })}
        width={720}
      >
        <Table
          rowKey="id"
          size="small"
          dataSource={runsDrawer.runs}
          pagination={{ pageSize: 20 }}
          columns={[
            { title: 'Run ID', dataIndex: 'id', width: 70 },
            { title: '状态', dataIndex: 'status', width: 90, render: (s: string) => <Tag color={statusColor[s]}>{s}</Tag> },
            { title: '触发', dataIndex: 'trigger_type', width: 80 },
            { title: '开始', dataIndex: 'started_at', width: 180 },
            { title: '完成', dataIndex: 'finished_at', width: 180, render: (v: string | null) => v || '-' },
            { title: '记录数', dataIndex: 'records_affected', width: 80 },
            { title: '错误', dataIndex: 'error', render: (v: string | null) => v || '-' },
          ]}
        />
      </Drawer>
    </div>
  )
}
