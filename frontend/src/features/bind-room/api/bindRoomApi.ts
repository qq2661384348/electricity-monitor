import { bindingApi } from '@/entities/binding';
import { roomApi } from '@/entities/room';

export const bindRoomApi = {
  createBinding: bindingApi.createBinding,
  queryPathTree: roomApi.queryPathTree,
};
