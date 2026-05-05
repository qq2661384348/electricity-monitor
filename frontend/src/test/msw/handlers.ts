import { http, HttpResponse } from 'msw';

import type { Binding, LoginResponse, PathChildrenResponse, Room, RoomByPathResponse, User } from '@/types';

const defaultUser: User = {
  id: 'user-1',
  qq_number: '123456789',
  role: 'user',
  is_active: true,
};

const defaultRoom: Room = {
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
};

const defaultBinding: Binding = {
  id: 'binding-1',
  user_id: defaultUser.id,
  roomid: defaultRoom.roomid,
  notification_enabled: true,
  created_at: '2026-03-25T00:00:00Z',
  updated_at: '2026-03-25T00:00:00Z',
  room: defaultRoom,
};

const defaultLoginResponse: LoginResponse = {
  access_token: 'access-token',
  token_type: 'Bearer',
  expires_in: 3600,
  user: defaultUser,
};

const rootPathTree: PathChildrenResponse = {
  children: [{ name: '校区A', is_leaf: false, room_count: 1 }],
  current_level: 0,
  total_count: 1,
};

const buildingPathTree: PathChildrenResponse = {
  children: [{ name: '101', is_leaf: true, room_count: 1, roomid: defaultRoom.roomid }],
  current_level: 1,
  total_count: 1,
};

const roomByPath: RoomByPathResponse = {
  roomid: defaultRoom.roomid,
  room_name: defaultRoom.room_name,
  electricity_fee: defaultRoom.electricity_fee,
  threshold: defaultRoom.threshold,
  primary_roompath: defaultRoom.primary_roompath,
};

export const handlers = [
  http.post('*/api/auth/send-verification-code', () =>
    HttpResponse.json({
      message: '验证码已发送',
      qq_number: defaultUser.qq_number,
    }),
  ),

  http.post('*/api/auth/verify-and-login', async ({ request }) => {
    const body = (await request.json()) as { qq_number?: string };

    return HttpResponse.json({
      ...defaultLoginResponse,
      user: {
        ...defaultLoginResponse.user,
        qq_number: body.qq_number ?? defaultLoginResponse.user.qq_number,
      },
    });
  }),

  http.post('*/api/auth/refresh', () =>
    HttpResponse.json(
      {
        error: '未登录',
        message: '缺少 refresh cookie',
      },
      { status: 401 },
    ),
  ),

  http.post('*/api/auth/logout', () => new HttpResponse(null, { status: 204 })),

  http.get('*/api/bindings', () => HttpResponse.json([defaultBinding])),

  http.post('*/api/bindings', async ({ request }) => {
    const body = (await request.json()) as { roomid?: number };

    return HttpResponse.json(
      {
        ...defaultBinding,
        roomid: body.roomid ?? defaultBinding.roomid,
      },
      { status: 201 },
    );
  }),

  http.put('*/api/bindings/:bindingId/notification', async ({ params, request }) => {
    const body = (await request.json()) as { notification_enabled?: boolean };

    return HttpResponse.json({
      ...defaultBinding,
      id: String(params.bindingId),
      notification_enabled: body.notification_enabled ?? defaultBinding.notification_enabled,
    });
  }),

  http.delete('*/api/bindings/:bindingId', () => new HttpResponse(null, { status: 204 })),

  http.get('*/api/rooms/path-tree', ({ request }) => {
    const url = new URL(request.url);
    const parent = url.searchParams.get('parent') ?? '';

    return HttpResponse.json(parent === '' ? rootPathTree : buildingPathTree);
  }),

  http.get('*/api/rooms/by-path', () => HttpResponse.json(roomByPath)),

  http.put('*/api/rooms/:roomId/threshold', async ({ params, request }) => {
    const body = (await request.json()) as { threshold?: number };

    return HttpResponse.json({
      ...defaultRoom,
      id: String(params.roomId),
      threshold: body.threshold ?? defaultRoom.threshold,
    });
  }),
];
