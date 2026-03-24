import { useQuery } from '@tanstack/react-query';
import { roomApi } from '@/entities/room';
import { roomKeys } from '@/shared/api/queryKeys';
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
    queryKey: roomKeys.pathTree(parent),
    
    queryFn: () => roomApi.queryPathTree(parent),
    enabled,
    staleTime: 5 * 60 * 1000,
    gcTime: 30 * 60 * 1000,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });
}
