import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from './store/authStore'
import Layout from './components/Layout'
import LoginPage from './pages/LoginPage'
import OverviewPage from './pages/OverviewPage'
import EndpointsPage from './pages/EndpointsPage'
import IncidentsPage from './pages/IncidentsPage'
import IncidentDetailPage from './pages/IncidentDetailPage'
import CommandsPage from './pages/CommandsPage'

function PrivateRoute({ children }: { children: React.ReactNode }) {
    const token = useAuthStore(s => s.token)
    return token ? <>{children}</> : <Navigate to="/login" replace />
}

export default function App() {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/login" element={<LoginPage />} />
                <Route path="/" element={<PrivateRoute><Layout /></PrivateRoute>}>
                    <Route index element={<OverviewPage />} />
                    <Route path="endpoints" element={<EndpointsPage />} />
                    <Route path="incidents" element={<IncidentsPage />} />
                    <Route path="incidents/:id" element={<IncidentDetailPage />} />
                    <Route path="commands" element={<CommandsPage />} />
                </Route>
            </Routes>
        </BrowserRouter>
    )
}
