import { useEffect, useState } from 'react'
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts'
import api from '../api/client'

interface Event {
    id: number
    hostname: string
    eventType: string
    processName: string
    riskScore: number
    severity: 'INFO' | 'SUSPICIOUS' | 'BLOCKED'
    timestamp: string
    actionTaken: string
}

interface Endpoint {
    id: number
    hostname: string
    riskState: string
    currentRiskScore: number
    status: string
}

export default function OverviewPage() {
    const [events, setEvents] = useState<Event[]>([])
    const [endpoints, setEndpoints] = useState<Endpoint[]>([])

    useEffect(() => {
        api.get('/events').then(r => setEvents(r.data)).catch(() => { })
        api.get('/endpoints').then(r => setEndpoints(r.data)).catch(() => { })
        const interval = setInterval(() => {
            api.get('/events').then(r => setEvents(r.data)).catch(() => { })
            api.get('/endpoints').then(r => setEndpoints(r.data)).catch(() => { })
        }, 5000)
        return () => clearInterval(interval)
    }, [])

    const blocked = endpoints.filter(e => e.riskState === 'BLOCKED').length
    const suspicious = endpoints.filter(e => e.riskState === 'SUSPICIOUS').length
    const online = endpoints.filter(e => e.status === 'ONLINE').length

    // Build risk chart from recent events
    const chartData = events.slice(0, 20).reverse().map((e, i) => ({
        i,
        score: e.riskScore ?? 0,
        time: new Date(e.timestamp).toLocaleTimeString(),
    }))

    return (
        <div className="p-8 space-y-6">
            <div>
                <h1 className="text-2xl font-bold text-white">Overview</h1>
                <p className="text-sm text-gray-500 mt-1">Real-time threat posture</p>
            </div>

            {/* Metric cards */}
            <div className="grid grid-cols-4 gap-4">
                {[
                    { label: 'Endpoints Online', value: online, color: 'text-success' },
                    { label: 'Under Surveillance', value: suspicious, color: 'text-warning' },
                    { label: 'Auto-Blocked Today', value: blocked, color: 'text-danger' },
                    { label: 'Events (last 50)', value: events.length, color: 'text-accent' },
                ].map(({ label, value, color }) => (
                    <div key={label} className="card">
                        <p className="text-xs text-gray-500 font-medium uppercase tracking-wide">{label}</p>
                        <p className={`text-3xl font-bold mt-2 ${color}`}>{value}</p>
                    </div>
                ))}
            </div>

            {/* Risk Score Chart */}
            <div className="card">
                <h2 className="text-sm font-semibold text-gray-400 mb-4">Risk Score Timeline (last 20 events)</h2>
                <ResponsiveContainer width="100%" height={200}>
                    <LineChart data={chartData}>
                        <XAxis dataKey="time" tick={{ fill: '#6b7280', fontSize: 10 }} />
                        <YAxis domain={[0, 100]} tick={{ fill: '#6b7280', fontSize: 10 }} />
                        <Tooltip
                            contentStyle={{ background: '#111827', border: '1px solid #1f2937', borderRadius: '8px', fontSize: '12px' }}
                            labelStyle={{ color: '#9ca3af' }}
                            itemStyle={{ color: '#06b6d4' }}
                        />
                        <Line type="monotone" dataKey="score" stroke="#06b6d4" strokeWidth={2} dot={false} />
                    </LineChart>
                </ResponsiveContainer>
            </div>

            {/* Recent Events */}
            <div className="card">
                <h2 className="text-sm font-semibold text-gray-400 mb-4">Recent Events</h2>
                <div className="space-y-2">
                    {events.slice(0, 8).map(ev => (
                        <div key={ev.id} className="flex items-center gap-3 p-3 bg-bg rounded-lg border border-border">
                            <span className={ev.severity === 'BLOCKED' ? 'badge-blocked' : ev.severity === 'SUSPICIOUS' ? 'badge-suspicious' : 'badge-clean'}>
                                {ev.severity}
                            </span>
                            <span className="font-mono text-xs text-gray-300 flex-1">{ev.hostname}</span>
                            <span className="text-xs text-gray-400">{ev.processName}</span>
                            <span className="text-xs text-accent font-mono">{ev.riskScore?.toFixed(0)}</span>
                            <span className="text-xs text-gray-600">{new Date(ev.timestamp).toLocaleTimeString()}</span>
                        </div>
                    ))}
                    {events.length === 0 && (
                        <p className="text-sm text-gray-600 text-center py-4">No events yet. Waiting for agent telemetry...</p>
                    )}
                </div>
            </div>
        </div>
    )
}
