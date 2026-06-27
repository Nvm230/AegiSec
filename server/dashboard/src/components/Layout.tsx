import { Outlet, NavLink, useNavigate } from 'react-router-dom'
import { useAuthStore } from '../store/authStore'
import { ShieldCheckIcon, ServerIcon, BellAlertIcon, CommandLineIcon, ArrowRightStartOnRectangleIcon } from '@heroicons/react/24/outline'

const nav = [
    { to: '/', label: 'Overview', icon: ShieldCheckIcon, exact: true },
    { to: '/endpoints', label: 'Endpoints', icon: ServerIcon },
    { to: '/incidents', label: 'Incidents', icon: BellAlertIcon },
    { to: '/commands', label: 'Commands', icon: CommandLineIcon },
]

export default function Layout() {
    const { username, logout } = useAuthStore()
    const navigate = useNavigate()

    function handleLogout() {
        logout()
        navigate('/login')
    }

    return (
        <div className="flex h-screen overflow-hidden">
            {/* Sidebar */}
            <aside className="w-60 bg-surface border-r border-border flex flex-col py-6 px-4">
                {/* Logo */}
                <div className="flex items-center gap-3 mb-10 px-2">
                    <div className="w-8 h-8 bg-accent rounded-lg flex items-center justify-center">
                        <ShieldCheckIcon className="w-5 h-5 text-white" />
                    </div>
                    <div>
                        <div className="font-bold text-white text-sm">AegiSec</div>
                        <div className="text-xs text-gray-500">v2.0</div>
                    </div>
                </div>

                {/* Nav */}
                <nav className="flex-1 space-y-1">
                    {nav.map(({ to, label, icon: Icon, exact }) => (
                        <NavLink
                            key={to}
                            to={to}
                            end={exact}
                            className={({ isActive }) =>
                                `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ` +
                                (isActive
                                    ? 'bg-accent/15 text-accent'
                                    : 'text-gray-400 hover:text-white hover:bg-white/5')
                            }
                        >
                            <Icon className="w-4 h-4" />
                            {label}
                        </NavLink>
                    ))}
                </nav>

                {/* User */}
                <div className="border-t border-border pt-4 mt-4">
                    <div className="flex items-center justify-between">
                        <div>
                            <p className="text-sm font-medium text-white">{username}</p>
                            <p className="text-xs text-gray-500">Analyst</p>
                        </div>
                        <button onClick={handleLogout} title="Logout"
                            className="text-gray-400 hover:text-danger transition-colors">
                            <ArrowRightStartOnRectangleIcon className="w-5 h-5" />
                        </button>
                    </div>
                </div>
            </aside>

            {/* Main */}
            <main className="flex-1 overflow-y-auto">
                <Outlet />
            </main>
        </div>
    )
}
