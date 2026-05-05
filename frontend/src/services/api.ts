import httpClient from '@/shared/api/http-client';

export { authApi } from '@/features/auth-login';
export { bindingApi } from '@/entities/binding';
export { publicConfigApi } from '@/entities/public-config';
export { roomApi } from '@/entities/room';
export { bindingKeys, publicConfigKeys, roomKeys } from '@/shared/api/queryKeys';

export default httpClient;
