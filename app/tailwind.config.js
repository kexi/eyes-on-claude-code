/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        'bg-primary': 'rgba(26, 26, 46, 0.85)',
        'bg-secondary': 'rgba(22, 33, 62, 0.9)',
        'bg-card': 'rgba(15, 52, 96, 0.9)',
        'text-primary': '#eaeaea',
        'text-secondary': '#a0a0a0',
        accent: '#e94560',
        success: '#4ade80',
        warning: '#fbbf24',
        info: '#60a5fa',
      },
      animation: {
        'pulse-slow': 'pulse-slow 1.5s infinite',
      },
      keyframes: {
        'pulse-slow': {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.5' },
        },
      },
    },
  },
  plugins: [],
};
