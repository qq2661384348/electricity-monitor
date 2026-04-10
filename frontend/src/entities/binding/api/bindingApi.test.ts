import { http, HttpResponse } from 'msw';
import { describe, expect, it } from 'vitest';

import { useAuthStore } from '@/stores/authStore';
import { server } from '@/test/msw/server';

import { bindingApi } from './bindingApi';

describe('bindingApi auth refresh flow', () => {
  it('refreshes access token once and replays concurrent unauthorized requests', async () => {
    let bindingRequestCount = 0;
    let refreshRequestCount = 0;

    server.use(
      http.get('*/api/bindings', () => {
        bindingRequestCount += 1;

        if (bindingRequestCount <= 2) {
          return HttpResponse.json(
            {
              error: '认证失败',
              message: 'access token 已过期',
            },
            { status: 401 },
          );
        }

        return HttpResponse.json([
          {
            id: 'binding-1',
            user_id: 'user-1',
            roomid: 1001,
            notification_enabled: true,
            created_at: '2026-03-25T00:00:00Z',
            updated_at: '2026-03-25T00:00:00Z',
            room: {
              id: 'room-1',
              roomid: 1001,
              electricity_fee: 23.5,
              send_flag: false,
              threshold: 30,
              room_name: '101',
              primary_roompath: '校区A/宿舍楼A/1楼/101',
              has_additional_paths: false,
              is_active: true,
              source_type: 'crawler',
              created_at: '2026-03-25T00:00:00Z',
              updated_at: '2026-03-25T00:00:00Z',
            },
          },
        ]);
      }),
      http.post('*/api/auth/refresh', () => {
        refreshRequestCount += 1;

        return HttpResponse.json({
          access_token: 'fresh-access-token',
          token_type: 'Bearer',
          expires_in: 3600,
          user: {
            id: 'user-1',
            qq_number: '123456789',
            role: 'user',
            is_active: true,
          },
        });
      }),
    );

    useAuthStore.setState({
      user: {
        id: 'user-1',
        qq_number: '123456789',
        role: 'user',
        is_active: true,
      },
      token: 'expired-access-token',
      isAuthenticated: true,
      isSessionReady: true,
    });

    const [firstResult, secondResult] = await Promise.all([
      bindingApi.getMyBindings(),
      bindingApi.getMyBindings(),
    ]);

    expect(firstResult).toHaveLength(1);
    expect(secondResult).toHaveLength(1);
    expect(refreshRequestCount).toBe(1);
    expect(bindingRequestCount).toBe(4);
    expect(useAuthStore.getState().token).toBe('fresh-access-token');
  });
});
