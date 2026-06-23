import httpClient from '@/shared/api/http-client';
import type { Binding } from '@/types';

export const bindingApi = {
  async getMyBindings(): Promise<Binding[]> {
    const { data } = await httpClient.get<Binding[]>('/bindings');
    return data;
  },

  async createBinding(roomid: string): Promise<Binding> {
    const { data } = await httpClient.post<Binding>('/bindings', { roomid });
    return data;
  },

  async deleteBinding(id: string): Promise<void> {
    await httpClient.delete(`/bindings/${id}`);
  },

  async updateNotificationEnabled(id: string, enabled: boolean): Promise<Binding> {
    const { data } = await httpClient.put<Binding>(`/bindings/${id}/notification`, {
      notification_enabled: enabled,
    });
    return data;
  },
};
