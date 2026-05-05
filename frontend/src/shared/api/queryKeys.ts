export const bindingKeys = {
  all: ['bindings'] as const,
  detail: (bindingId: string) => ['bindings', bindingId] as const,
};

export const roomKeys = {
  all: ['rooms'] as const,
  list: (limit = 20, offset = 0) => ['rooms', 'list', limit, offset] as const,
  detail: (roomId: string) => ['rooms', 'detail', roomId] as const,
  byRoomId: (roomid: number) => ['rooms', 'roomid', roomid] as const,
  flagged: () => ['rooms', 'flagged'] as const,
  pathTree: (parent = '') => ['rooms', 'path-tree', parent] as const,
  byPath: (path: string) => ['rooms', 'path', path] as const,
};

export const publicConfigKeys = {
  all: ['public-config'] as const,
};
