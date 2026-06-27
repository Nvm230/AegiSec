import { useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import { Client } from '@stomp/stompjs'
import SockJS from 'sockjs-client'
import api from '../api/client'

interface Event {
    id: number
    hostname: string
    eventType: string
    processName: string
    cmdline: string
    riskScore: number
    severity: 'INFO' | 'SUSPICIOUS' | 'BLOCKED'
    timestamp: string
    actionTaken: string
}

export default function IncidentsPage() {
    const [events, setEvents] = useState<Event[]>([])
    const clientRef = useRef<Client | null>(null)

    useEffect(() => {
        // Load initial events
        api.get('/events').then(r => setEvents(r.data)).catch(() => { })

        // Subscribe to live WebSocket alerts
        const client = new Client({
            webSocketFactory: () => new SockJS('/ws'),
            reconnectDelay: 3000,
            onConnect: () => {
                client.subscribe('/topic/alerts', msg => {
                    const ev: Event = JSON.parse(msg.body)
                    setEvents(prev => [ev, ...prev].slice(0, 100))
                })
            },
        })
        client.activate()
        clientRef.current = client
        return () => { client.deactivate() }
    }, [])

    function severityStyles(sev: string) {
        switch (sev) {
            case 'BLOCKED': return 'border-l-danger bg-danger/5'
            case 'SUSPICIOUS': return 'border-l-warning bg-warning/5'
            default: return 'border-l-gray-700 bg-bg'
        }
    }

    return (
        <div className="p-8">
            <div className="flex items-center justify-between mb-6">
                <div>
                    <h1 className="text-2xl font-bold text-white">Incident Feed</h1>
                    <p className="text-sm text-gray-500 mt-1">Live stream via WebSocket</p>
                </div>
                <div className="flex items-center gap-2">
                    <span className="w-2 h-2 bg-success rounded-full animate-pulse" />
                    <span className="text-xs text-success">Live</span>
                </div>
            </div>

            <div className="space-y-2">
                {events.map(ev => (
                    <Link key={ev.id} to={`/incidents/${ev.id}`}
                        className={`block card border-l-4 ${severityStyles(ev.severity)} hover:border-accent transition-colors p-4`}>
                        <div className="flex items-start justify-between gap-4">
                            <div className="flex-1 min-w-0">
                                <div className="flex items-center gap-2 mb-1">
                                    <span className={ev.severity === 'BLOCKED' ? 'badge-blocked' : ev.severity === 'SUSPICIOUS' ? 'badge-suspicious' : 'badge-clean'}>
                                        {ev.severity}
                                    </span>
                                    <span className="text-xs text-gray-500">{ev.eventType}</span>
                                    <span className="text-xs text-gray-600">{ev.hostname}</span>
                                </div>
                                {ev.cmdline && (
                                    <p className="font-mono text-xs text-gray-300 bg-black/20 px-2 py-1 rounded mt-1 truncate">
                                        {ev.cmdline}
                                    </p>
                                )}
                                {ev.actionTaken && ev.actionTaken !== 'NONE' && (
                                    <p className="text-xs text-accent mt-1">⚡ {ev.actionTaken}</p>
                                )}
                            </div>
                            <div className="text-right shrink-0">
                                <p className="text-xl font-bold font-mono" style={{
                                    color: ev.riskScore >= 80 ? '#ef4444' : ev.riskScore >= 40 ? '#f59e0b' : '#10b981'
                                }}>{ev.riskScore?.toFixed(0)}</p>
                                <p className="text-xs text-gray-600">{new Date(ev.timestamp).toLocaleTimeString()}</p>
                            </div>
                        </div>
                    </Link>
                ))}
                {events.length === 0 && (
                    <div className="text-center text-gray-500 py-20">
                        <p className="text-4xl mb-3">🛡️</p>
                        <p>All clear. No suspicious events detected.</p>
                    </div>
                )}
            </div>
        </div>
    )
}
