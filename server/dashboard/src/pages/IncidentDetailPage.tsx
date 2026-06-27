import { useEffect, useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import api from '../api/client'

interface Event {
    id: number
    hostname: string
    eventType: string
    processName: string
    cmdline: string
    riskScore: number
    severity: string
    timestamp: string
    actionTaken: string
    osInfo: string
    agentId: string
}

export default function IncidentDetailPage() {
    const { id } = useParams()
    const [event, setEvent] = useState<Event | null>(null)

    useEffect(() => {
        api.get(`/events`).then(r => {
            const ev = r.data.find((e: Event) => String(e.id) === id)
            setEvent(ev || null)
        }).catch(() => { })
    }, [id])

    if (!event) return (
        <div className="p-8 text-gray-500">Loading incident details...</div>
    )

    return (
        <div className="p-8 space-y-6 max-w-3xl">
            <div className="flex items-center gap-3">
                <Link to="/incidents" className="text-gray-500 hover:text-white transition-colors text-sm">
                    ← Incidents
                </Link>
                <span className="text-gray-700">/</span>
                <span className="text-sm text-gray-400">Incident #{id}</span>
            </div>

            <div className="flex items-start justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-white">{event.eventType}</h1>
                    <p className="text-sm text-gray-500 mt-1">{event.hostname} · {new Date(event.timestamp).toLocaleString()}</p>
                </div>
                <span className={event.severity === 'BLOCKED' ? 'badge-blocked text-base px-3 py-1.5' : 'badge-suspicious text-base px-3 py-1.5'}>
                    {event.severity}
                </span>
            </div>

            {/* Evidence */}
            <div className="card">
                <h2 className="text-sm font-semibold text-gray-400 mb-3">📋 Execution Evidence</h2>
                <div className="space-y-3">
                    <div className="grid grid-cols-2 gap-3 text-sm">
                        <div><span className="text-gray-500">Process</span><p className="text-white font-mono mt-0.5">{event.processName}</p></div>
                        <div><span className="text-gray-500">Risk Score</span><p className="font-bold font-mono mt-0.5" style={{ color: event.riskScore >= 80 ? '#ef4444' : '#f59e0b' }}>{event.riskScore?.toFixed(1)}</p></div>
                        <div><span className="text-gray-500">Agent ID</span><p className="text-white font-mono text-xs mt-0.5 truncate">{event.agentId}</p></div>
                        <div><span className="text-gray-500">OS</span><p className="text-white mt-0.5">{event.osInfo}</p></div>
                    </div>
                    {event.cmdline && (
                        <div>
                            <span className="text-gray-500 text-sm">Command Line</span>
                            <pre className="mt-1.5 bg-black/40 rounded-lg p-3 font-mono text-xs text-green-400 overflow-x-auto border border-border">
                                {event.cmdline}
                            </pre>
                        </div>
                    )}
                </div>
            </div>

            {/* Action Taken */}
            <div className="card">
                <h2 className="text-sm font-semibold text-gray-400 mb-3">⚡ AegiSec Response</h2>
                <div className={`flex items-center gap-3 p-3 rounded-lg ${event.actionTaken !== 'NONE' ? 'bg-success/10 border border-success/30' : 'bg-border/30'}`}>
                    <span className="text-2xl">{event.actionTaken !== 'NONE' ? '✅' : '⏭️'}</span>
                    <div>
                        <p className="text-sm font-medium text-white">{event.actionTaken !== 'NONE' ? event.actionTaken : 'No action required'}</p>
                        <p className="text-xs text-gray-500 mt-0.5">
                            {event.actionTaken === 'KILL_PROCESS' && 'Process terminated via SIGKILL'}
                            {event.actionTaken === 'BLOCK_IP' && 'IP blocked via nftables rule'}
                            {event.actionTaken === 'NONE' && 'Score below blocking threshold'}
                        </p>
                    </div>
                </div>
            </div>
        </div>
    )
}
