import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'node:path'

/**
 * 按 npm 包名分割 chunk
 * 
 * 分割策略：
 * 1. 核心框架库单独分割（react、react-dom）
 * 2. 大型 UI 库单独分割（framer-motion、lucide）
 * 3. 数据层库单独分割（tanstack-query、axios、zustand）
 * 4. 路由库单独分割（react-router）
 * 5. 其他小依赖合并到 vendor
 */
function manualChunks(id: string): string | undefined {
  if (!id.includes('node_modules')) {
    return undefined
  }
  
  // 提取包名
  const parts = id.split('node_modules/')[1]?.split('/')
  if (!parts) return 'vendor'
  
  // 处理 @scope/package 格式
  const packageName = parts[0].startsWith('@') 
    ? `${parts[0]}/${parts[1]}` 
    : parts[0]
  
  // React 核心（约 6KB + 130KB）
  if (id.includes('react-dom')) return 'lib-react-dom'
  if (id.includes('/react/') || packageName === 'react') return 'lib-react'
  if (id.includes('scheduler')) return 'lib-react'
  
  // 动画库 framer-motion（约 160KB）
  if (id.includes('framer-motion')) return 'lib-framer-motion'
  
  // 图标库 lucide（按需加载，约 50-100KB）
  if (id.includes('lucide')) return 'lib-lucide'
  
  // 数据获取 tanstack-query（约 40KB）
  if (id.includes('@tanstack')) return 'lib-tanstack'
  
  // 路由 react-router（约 30KB）
  if (id.includes('react-router')) return 'lib-react-router'
  
  // HTTP 客户端 axios（约 15KB）
  if (id.includes('axios')) return 'lib-axios'
  
  // 状态管理 zustand（约 3KB）
  if (id.includes('zustand')) return 'lib-zustand'
  
  // Tailwind 相关
  if (id.includes('tailwind') || id.includes('clsx') || id.includes('class-variance')) {
    return 'lib-styling'
  }
  
  // 其他小依赖合并到 vendor
  return 'vendor'
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [['babel-plugin-react-compiler']],
      },
    }),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    // 64KB 警告阈值（核心库如 react-dom 可超出）
    chunkSizeWarningLimit: 64,
    rollupOptions: {
      output: {
        manualChunks,
        // 优化 chunk 文件名格式
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash].[ext]',
      },
    },
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8000',
        changeOrigin: true,
      },
    },
  },
})
