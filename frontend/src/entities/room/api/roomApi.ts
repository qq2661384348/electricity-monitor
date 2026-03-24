import httpClient from '@/shared/api/http-client';
import type { PathChildrenResponse, Room, RoomByPathResponse } from '@/types';

export const roomApi = {
  async getRooms(limit = 20, offset = 0): Promise<Room[]> {
    const { data } = await httpClient.get<Room[]>('/rooms', {
      params: { limit, offset },
    });
    return data;
  },

  async getRoomByRoomId(roomid: number): Promise<Room> {
    const { data } = await httpClient.get<Room>(`/rooms/by-roomid/${roomid}`);
    return data;
  },

  async getFlaggedRooms(): Promise<Room[]> {
    const { data } = await httpClient.get<Room[]>('/rooms/flagged');
    return data;
  },

  async updateThreshold(id: string, threshold: number): Promise<Room> {
    const { data } = await httpClient.put<Room>(`/rooms/${id}/threshold`, { threshold });
    return data;
  },

  async queryPathTree(parent = ''): Promise<PathChildrenResponse> {
    const { data } = await httpClient.get<PathChildrenResponse>('/rooms/path-tree', {
      params: { parent },
    });
    return data;
  },

  async getRoomByPath(path: string): Promise<RoomByPathResponse> {
    const { data } = await httpClient.get<RoomByPathResponse>('/rooms/by-path', {
      params: { path },
    });
    return data;
  },
};
