import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'node:path'

/**
 * 按前端依赖包名分割 chunk。
 *
 * 当前策略只继续拆分可拆的公共依赖，
 * 启动期核心库的真实体积由 check:bundle 做细粒度预算约束。
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
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    css: true,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    // 192KB 仅作为 Vite 的粗粒度 warning 阈值。
    // 真正的 chunk 预算由 scripts/check-bundle-budgets.ts 统一校验。
    chunkSizeWarningLimit: 192,
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
