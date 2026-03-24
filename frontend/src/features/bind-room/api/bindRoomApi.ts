import { bindingApi } from '@/entities/binding';
import { roomApi } from '@/entities/room';

export const bindRoomApi = {
  createBinding: bindingApi.createBinding,
  getRoomByPath: roomApi.getRoomByPath,
  queryPathTree: roomApi.queryPathTree,
};
