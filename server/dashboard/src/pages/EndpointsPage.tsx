import { useEffect, useState } from 'react'
import api from '../api/client'

interface Endpoint {
    id: number
    agentId: string
    hostname: string
    ip: string
    osInfo: string
    status: 'ONLINE' | 'OFFLINE'
    riskState: 'CLEAN' | 'SUSPICIOUS' | 'BLOCKED'
    currentRiskScore: number
    lastPing: string
}

export default function EndpointsPage() {
    const [endpoints, setEndpoints] = useState<Endpoint[]>([])

    useEffect(() => {
        const fetch = () => api.get('/endpoints').then(r => setEndpoints(r.data)).catch(() => { })
        fetch()
        const id = setInterval(fetch, 5000)
        return () => clearInterval(id)
    }, [])

    return (
        <div className="p-8">
            <h1 className="text-2xl font-bold text-white mb-1">Endpoints</h1>
            <p className="text-sm text-gray-500 mb-6">Monitored host inventory</p>

            <div className="card overflow-hidden p-0">
                <table className="w-full text-sm">
                    <thead className="bg-bg text-gray-500 text-xs uppercase tracking-wide">
                        <tr>
                            {['Hostname', 'IP', 'OS', 'Status', 'Risk State', 'Score', 'Last Ping'].map(h => (
                                <th key={h} className="px-4 py-3 text-left font-medium">{h}</th>
                            ))}
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-border">
                        {endpoints.map(ep => (
                            <tr key={ep.id} className="hover:bg-white/[0.02] transition-colors">
                                <td className="px-4 py-3 font-medium text-white">{ep.hostname || '—'}</td>
                                <td className="px-4 py-3 font-mono text-gray-400">{ep.ip || '—'}</td>
                                <td className="px-4 py-3 text-gray-400">{ep.osInfo || '—'}</td>
                                <td className="px-4 py-3">
                                    <span className={`inline-flex items-center gap-1.5 text-xs font-medium ${ep.status === 'ONLINE' ? 'text-success' : 'text-gray-500'}`}>
                                        <span className={`w-1.5 h-1.5 rounded-full ${ep.status === 'ONLINE' ? 'bg-success' : 'bg-gray-500'}`} />
                                        {ep.status}
                                    </span>
                                </td>
                                <td className="px-4 py-3">
                                    <span className={
                                        ep.riskState === 'BLOCKED' ? 'badge-blocked' :
                                            ep.riskState === 'SUSPICIOUS' ? 'badge-suspicious' : 'badge-clean'
                                    }>
                                        {ep.riskState}
                                    </span>
                                </td>
                                <td className="px-4 py-3">
                                    <div className="flex items-center gap-2">
                                        <div className="flex-1 bg-border rounded-full h-1.5 w-20">
                                            <div
                                                className="h-1.5 rounded-full transition-all"
                                                style={{
                                                    width: `${Math.min(ep.currentRiskScore, 100)}%`,
                                                    background: ep.currentRiskScore >= 80 ? '#ef4444' : ep.currentRiskScore >= 40 ? '#f59e0b' : '#10b981'
                                                }}
                                            />
                                        </div>
                                        <span className="font-mono text-xs text-gray-400">{ep.currentRiskScore?.toFixed(0)}</span>
                                    </div>
                                </td>
                                <td className="px-4 py-3 text-xs text-gray-500">
                                    {ep.lastPing ? new Date(ep.lastPing).toLocaleTimeString() : '—'}
                                </td>
                            </tr>
                        ))}
                        {endpoints.length === 0 && (
                            <tr><td colSpan={7} className="px-4 py-10 text-center text-gray-600">No endpoints online yet.</td></tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    )
}
