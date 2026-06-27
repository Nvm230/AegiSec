import axios from 'axios'
import { useAuthStore } from '../store/authStore'

const api = axios.create({
    baseURL: '/api/v1',
})

// Attach JWT token to every request
api.interceptors.request.use(config => {
    const token = useAuthStore.getState().token
    if (token) {
        config.headers.Authorization = `Bearer ${token}`
    }
    return config
})

// Auto-logout on 401
api.interceptors.response.use(
    r => r,
    err => {
        if (err.response?.status === 401) {
            useAuthStore.getState().logout()
        }
        return Promise.reject(err)
    }
)

export default api
