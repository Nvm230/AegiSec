import { useEffect, useState } from 'react'
import api from '../api/client'

interface Endpoint {
    id: number
    hostname: string
    riskState: string
}

export default function CommandsPage() {
    const [endpoints, setEndpoints] = useState<Endpoint[]>([])
    const [message, setMessage] = useState<string | null>(null)

    useEffect(() => {
        api.get('/endpoints').then(r => setEndpoints(r.data)).catch(() => { })
    }, [])

    async function setState(id: number, state: 'CLEAN') {
        try {
            await api.put(`/endpoints/${id}/state?state=${state}`)
            setMessage(`✅ Endpoint unblocked successfully`)
            api.get('/endpoints').then(r => setEndpoints(r.data)).catch(() => { })
        } catch {
            setMessage('❌ Failed to execute command')
        }
        setTimeout(() => setMessage(null), 3000)
    }

    const blocked = endpoints.filter(e => e.riskState === 'BLOCKED')

    return (
        <div className="p-8">
            <h1 className="text-2xl font-bold text-white mb-1">Command Center</h1>
            <p className="text-sm text-gray-500 mb-6">Manual response actions for analysts</p>

            {message && (
                <div className="mb-4 text-sm bg-surface border border-border rounded-lg px-4 py-3">{message}</div>
            )}

            <div className="card mb-6">
                <h2 className="text-sm font-semibold text-gray-400 mb-4">Blocked Endpoints ({blocked.length})</h2>
                {blocked.length === 0 ? (
                    <p className="text-sm text-gray-600">No endpoints are currently blocked.</p>
                ) : (
                    <div className="space-y-2">
                        {blocked.map(ep => (
                            <div key={ep.id} className="flex items-center justify-between p-3 bg-bg rounded-lg border border-border">
                                <div>
                                    <p className="text-sm font-medium text-white">{ep.hostname}</p>
                                    <span className="badge-blocked mt-1 inline-block">{ep.riskState}</span>
                                </div>
                                <button
                                    id={`unblock-${ep.id}`}
                                    onClick={() => setState(ep.id, 'CLEAN')}
                                    className="btn-primary text-xs"
                                >
                                    Un-isolate Host
                                </button>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <div className="card">
                <h2 className="text-sm font-semibold text-gray-400 mb-4">Quick Actions</h2>
                <div className="grid grid-cols-3 gap-3">
                    {[
                        { label: '🔄 Force Refresh', desc: 'Reload all endpoint data', action: () => api.get('/endpoints').then(r => setEndpoints(r.data)) },
                        { label: '📊 Risk Report', desc: 'Download current risk summary', action: () => alert('Coming soon') },
                        { label: '🧹 Clear Scores', desc: 'Reset all risk scores (dev only)', action: () => alert('Coming soon') },
                    ].map(({ label, desc, action }) => (
                        <button key={label} onClick={action}
                            className="card text-left hover:border-accent/50 transition-colors cursor-pointer">
                            <p className="text-sm font-medium text-white">{label}</p>
                            <p className="text-xs text-gray-500 mt-1">{desc}</p>
                        </button>
                    ))}
                </div>
            </div>
        </div>
    )
}
