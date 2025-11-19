import { useQuery } from '@tanstack/react-query';
import { roomApi, bindingApi } from '@/services/api';
import { useAuthStore } from '@/stores/authStore';
import type { Room } from '@/types';

/**
 * 查询用户的绑定列表（Binding为核心）
 * Binding包含room信息，可直接用于显示
 */
export const useBindingsQuery = () => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  
  return useQuery({
    queryKey: ['bindings'],
    queryFn: () => bindingApi.getMyBindings(),
    enabled: isAuthenticated,
    staleTime: 5 * 60 * 1000, // 5分钟缓存
  });
};

/**
 * @deprecated 使用 useBindingsQuery 替代
 * 保留以兼容预警墙功能
 */
export const useRoomsQuery = () => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  
  return useQuery({
    queryKey: ['rooms', 'list'],
    queryFn: () => roomApi.getRooms(100, 0), // 获取更多房间
    enabled: isAuthenticated, // 只有登录后才请求
  });
};

export const useFlaggedRoomsQuery = () => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  return useQuery({
    queryKey: ['rooms', 'flagged'],
    queryFn: () => roomApi.getFlaggedRooms(),
    enabled: isAuthenticated,
    refetchInterval: 1000 * 60, // 每分钟轮询一次预警
  });
};

// 组合 Hook：计算统计数据
export const useDashboardStats = (rooms: Room[] = [], flaggedRooms: Room[] = []) => {
  return {
    totalRooms: rooms.length,
    flaggedRooms: flaggedRooms.length,
    averageFee: rooms.length > 0
      ? rooms.reduce((sum, r) => sum + r.electricity_fee, 0) / rooms.length
      : 0,
    todayConsumption: 0, // 暂时没有此数据
  };
};
