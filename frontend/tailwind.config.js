/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // 漫威风格主题色
        marvel: {
          red: '#E63946',      // Iron Man 红
          gold: '#FFD60A',     // 金黄能量
          'deep-blue': '#001D3D', // 深空蓝背景
          lightning: '#00B4D8',   // Thor 闪电蓝
          silver: '#A8DADC',      // 银色点缀
        },
        // 状态色彩
        status: {
          normal: '#06D6A0',    // 正常-绿
          warning: '#FFB703',   // 警告-黄
          danger: '#E63946',    // 危险-红
          critical: '#9B2226',  // 严重-深红
        },
      },
      backgroundImage: {
        'hero-gradient': 'linear-gradient(135deg, #E63946 0%, #FFD60A 100%)',
        'space-gradient': 'linear-gradient(180deg, #001D3D 0%, #051923 50%, #000814 100%)',
        'energy-pulse': 'radial-gradient(circle, #FFD60A 0%, transparent 70%)',
      },
      boxShadow: {
        'neon': '0 0 10px currentColor, 0 0 20px currentColor, 0 0 30px currentColor',
        'marvel': '0 0 20px rgba(255, 214, 10, 0.5), 0 0 40px rgba(230, 57, 70, 0.3)',
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'glow': 'glow 2s ease-in-out infinite alternate',
        'float': 'float 3s ease-in-out infinite',
        'slide-up': 'slideUp 0.5s ease-out',
      },
      keyframes: {
        glow: {
          '0%': { boxShadow: '0 0 5px currentColor' },
          '100%': { boxShadow: '0 0 20px currentColor, 0 0 30px currentColor' },
        },
        float: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        slideUp: {
          '0%': { transform: 'translateY(100px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}
