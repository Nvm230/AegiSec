/** @type {import('tailwindcss').Config} */
export default {
    content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
    theme: {
        extend: {
            fontFamily: {
                sans: ['Inter', 'sans-serif'],
                mono: ['JetBrains Mono', 'monospace'],
            },
            colors: {
                bg: '#0a0e1a',
                surface: '#111827',
                border: '#1f2937',
                accent: '#06b6d4',
                danger: '#ef4444',
                warning: '#f59e0b',
                success: '#10b981',
            },
        },
    },
    plugins: [],
}
