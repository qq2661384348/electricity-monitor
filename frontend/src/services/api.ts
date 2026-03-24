import httpClient from '@/shared/api/http-client';

export { authApi } from '@/features/auth-login';
export { bindingApi } from '@/entities/binding';
export { roomApi } from '@/entities/room';
export { bindingKeys, roomKeys } from '@/shared/api/queryKeys';

export default httpClient;
