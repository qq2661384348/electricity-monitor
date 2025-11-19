import { QueryClient } from '@tanstack/react-query';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5分钟内数据被认为是新鲜的
      gcTime: 1000 * 60 * 30,   // 30分钟后进行垃圾回收
      retry: 1,                 // 失败重试1次
      refetchOnWindowFocus: false, // 窗口聚焦时不重新获取
    },
    mutations: {
      retry: 0, // Mutation 默认不重试
    },
  },
});
