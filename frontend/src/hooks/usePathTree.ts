import { useQuery } from '@tanstack/react-query';
import { roomApi } from '@/services/api';
import type { PathChildrenResponse } from '@/types';

/**
 * 路径树查询 Hook - 使用 React Query 缓存和重试
 * 
 * @param parent - 父路径（空字符串表示根节点）
 * @param enabled - 是否启用查询（默认true）
 * @returns React Query 查询结果
 * 
 * @example
 * ```tsx
 * const { data, isLoading, error, refetch } = usePathTree('校区/建筑');
 * ```
 */
export function usePathTree(parent: string, enabled = true) {
  return useQuery<PathChildrenResponse>({
    // 查询键：根据父路径缓存
    queryKey: ['path-tree', parent],
    
    // 查询函数
    queryFn: () => roomApi.queryPathTree(parent),
    
    // 是否启用查询
    enabled,
    
    // 缓存策略：5分钟内数据被认为是新鲜的
    staleTime: 5 * 60 * 1000,
    
    // 垃圾回收时间：30分钟后回收
    gcTime: 30 * 60 * 1000,
    
    // 重试策略：失败后重试3次
    retry: 3,
    
    // 指数退避重试延迟：最大30秒
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    
    // 窗口聚焦时不重新获取
    refetchOnWindowFocus: false,
    
    // 重新连接时不重新获取
    refetchOnReconnect: false,
  });
}
