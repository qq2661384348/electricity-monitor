import { useQuery } from '@tanstack/react-query';

import { publicConfigApi } from '@/entities/public-config';
import { publicConfigKeys } from '@/shared/api/queryKeys';

export function usePublicConfig() {
  return useQuery({
    queryKey: publicConfigKeys.all,
    queryFn: () => publicConfigApi.getPublicConfig(),
    staleTime: 30 * 60 * 1000,
  });
}
