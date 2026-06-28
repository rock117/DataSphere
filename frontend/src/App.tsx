import {
  BrowserRouter,
  Routes,
  Route,
  Navigate,
  useNavigate,
  useLocation,
} from "react-router-dom";
import { Layout, Menu } from "antd";
import {
  ScheduleOutlined,
  StockOutlined,
  LineChartOutlined,
} from "@ant-design/icons";
import Tasks from "./pages/Tasks";
import Stocks from "./pages/Stocks";
import Klines from "./pages/Klines";

const { Header, Sider, Content } = Layout;

const menuItems = [
  { key: "/tasks", icon: <ScheduleOutlined />, label: "任务管理" },
  { key: "/stocks", icon: <StockOutlined />, label: "股票列表" },
  { key: "/klines", icon: <LineChartOutlined />, label: "行情查看" },
];

function AppLayout() {
  const navigate = useNavigate();
  const location = useLocation();
  return (
    <Layout style={{ minHeight: "100vh" }}>
      <Sider collapsible>
        <div
          style={{
            height: 48,
            color: "#fff",
            textAlign: "center",
            lineHeight: "48px",
            fontSize: 18,
            fontWeight: "bold",
          }}
        >
          DataSphere
        </div>
        <Menu
          theme="dark"
          mode="inline"
          items={menuItems}
          selectedKeys={[location.pathname]}
          onClick={({ key }) => navigate(key)}
        />
      </Sider>
      <Layout>
        <Header style={{ background: "#fff", padding: "0 24px" }}>
          <h3>金融数据收集系统</h3>
        </Header>
        <Content>
          <div className="page-content">
            <Routes>
              <Route path="/" element={<Navigate to="/tasks" replace />} />
              <Route path="/tasks" element={<Tasks />} />
              <Route path="/stocks" element={<Stocks />} />
              <Route path="/klines" element={<Klines />} />
            </Routes>
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <AppLayout />
    </BrowserRouter>
  );
}
