import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '../store/authStore'
import api from '../api/client'

export default function LoginPage() {
    const [username, setUsername] = useState('')
    const [password, setPassword] = useState('')
    const [error, setError] = useState('')
    const [loading, setLoading] = useState(false)
    const { login } = useAuthStore()
    const navigate = useNavigate()

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault()
        setError('')
        setLoading(true)
        try {
            const res = await api.post('/auth/login', { username, password })
            login(res.data.token, res.data.username, res.data.role)
            navigate('/')
        } catch {
            setError('Invalid credentials')
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className="min-h-screen bg-bg flex items-center justify-center">
            {/* Glow */}
            <div className="absolute inset-0 overflow-hidden pointer-events-none">
                <div className="absolute -top-40 left-1/2 -translate-x-1/2 w-[600px] h-[600px] bg-accent/5 rounded-full blur-3xl" />
            </div>

            <div className="w-full max-w-sm">
                {/* Logo */}
                <div className="text-center mb-10">
                    <div className="inline-flex items-center justify-center w-14 h-14 bg-accent/10 border border-accent/30 rounded-2xl mb-4">
                        <svg className="w-7 h-7 text-accent" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                                d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
                        </svg>
                    </div>
                    <h1 className="text-2xl font-bold text-white">AegiSec</h1>
                    <p className="text-sm text-gray-500 mt-1">Security Operations Dashboard</p>
                </div>

                {/* Form */}
                <form onSubmit={handleSubmit} className="card space-y-4">
                    <div>
                        <label className="text-xs font-medium text-gray-400 block mb-1.5">Username</label>
                        <input
                            id="username"
                            type="text"
                            value={username}
                            onChange={e => setUsername(e.target.value)}
                            className="w-full bg-bg border border-border rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:border-accent transition-colors"
                            placeholder="admin"
                            required
                        />
                    </div>
                    <div>
                        <label className="text-xs font-medium text-gray-400 block mb-1.5">Password</label>
                        <input
                            id="password"
                            type="password"
                            value={password}
                            onChange={e => setPassword(e.target.value)}
                            className="w-full bg-bg border border-border rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:border-accent transition-colors"
                            placeholder="••••••••"
                            required
                        />
                    </div>

                    {error && (
                        <p className="text-xs text-danger bg-danger/10 border border-danger/30 rounded-lg px-3 py-2">
                            {error}
                        </p>
                    )}

                    <button
                        id="login-btn"
                        type="submit"
                        disabled={loading}
                        className="btn-primary w-full justify-center flex"
                    >
                        {loading ? 'Authenticating...' : 'Sign In'}
                    </button>
                </form>

                <p className="text-center text-xs text-gray-600 mt-4">
                    Protected by mTLS · JWT authenticated
                </p>
            </div>
        </div>
    )
}
