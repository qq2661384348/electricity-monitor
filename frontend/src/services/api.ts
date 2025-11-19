import axios, { type AxiosError } from 'axios';
import { useAuthStore } from '@/stores/authStore';
import type { 
  User, 
  Room, 
  Binding, 
  LoginResponse, 
  ApiError,
  PathChildrenResponse,
  RoomByPathResponse
} from '@/types';

// 创建 Axios 实例
const api = axios.create({
  baseURL: '/api',
  timeout: 15000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器：添加认证token
api.interceptors.request.use(
  (config) => {
    const token = useAuthStore.getState().token;
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// 响应拦截器：统一错误处理
api.interceptors.response.use(
  (response) => response,
  (error: AxiosError<ApiError>) => {
    if (error.response?.status === 401) {
      // 未授权，清除token并提示登录
      useAuthStore.getState().logout();
    }
    
    return Promise.reject(error);
  }
);

// 认证相关API
export const authApi = {
  /**
   * 发送验证码
   * @param qqNumber QQ号码
   * @param captchaToken 验证码token（可选，后续将改为必须）
   */
  async sendVerificationCode(qqNumber: string, captchaToken?: string): Promise<void> {
    await api.post('/auth/send-verification-code', { 
      qq_number: qqNumber,
      captcha_token: captchaToken,
    });
  },

  /**
   * 验证并登录
   */
  async verifyAndLogin(qqNumber: string, code: string): Promise<LoginResponse> {
    const { data } = await api.post<LoginResponse>('/auth/verify-and-login', {
      qq_number: qqNumber,
      code,
    });
    return data;
  },

  /**
   * 刷新token
   */
  async refreshToken(refreshToken: string): Promise<LoginResponse> {
    const { data } = await api.post<LoginResponse>('/auth/refresh', {
      refresh_token: refreshToken,
    });
    return data;
  },

  /**
   * 获取当前用户信息
   */
  async getCurrentUser(): Promise<User> {
    const { data } = await api.get<User>('/auth/me');
    return data;
  },
};

// 房间相关API
export const roomApi = {
  /**
   * 获取房间列表（分页）
   */
  async getRooms(limit = 20, offset = 0): Promise<Room[]> {
    const { data } = await api.get<Room[]>('/rooms', {
      params: { limit, offset },
    });
    return data;
  },

  /**
   * 根据roomid获取房间详情
   */
  async getRoomByRoomId(roomid: number): Promise<Room> {
    const { data } = await api.get<Room>(`/rooms/by-roomid/${roomid}`);
    return data;
  },

  /**
   * 获取需要通知的房间（flagged）
   */
  async getFlaggedRooms(): Promise<Room[]> {
    const { data } = await api.get<Room[]>('/rooms/flagged');
    return data;
  },

  /**
   * 更新房间阈值
   */
  async updateThreshold(id: string, threshold: number): Promise<Room> {
    const { data } = await api.put<Room>(`/rooms/${id}/threshold`, { threshold });
    return data;
  },

  /**
   * 查询路径树子节点（逐层查询）
   * @param parent 父路径（空字符串表示查询根节点）
   */
  async queryPathTree(parent = ''): Promise<PathChildrenResponse> {
    const { data } = await api.get<PathChildrenResponse>('/rooms/path-tree', {
      params: { parent },
    });
    return data;
  },

  /**
   * 根据完整路径查询房间
   * @param path 完整路径（如 "箭盘校区/北区12栋/三楼/B12313"）
   */
  async getRoomByPath(path: string): Promise<RoomByPathResponse> {
    const { data } = await api.get<RoomByPathResponse>('/rooms/by-path', {
      params: { path },
    });
    return data;
  },
};

// 绑定关系相关API
export const bindingApi = {
  /**
   * 获取用户的所有绑定
   */
  async getMyBindings(): Promise<Binding[]> {
    const { data } = await api.get<Binding[]>('/bindings');
    return data;
  },

  /**
   * 创建绑定
   */
  async createBinding(roomid: number): Promise<Binding> {
    const { data } = await api.post<Binding>('/bindings', { roomid });
    return data;
  },

  /**
   * 删除绑定
   */
  async deleteBinding(id: string): Promise<void> {
    await api.delete(`/bindings/${id}`);
  },

  /**
   * 更新绑定的通知设置
   */
  async updateNotificationEnabled(
    id: string, 
    enabled: boolean
  ): Promise<Binding> {
    const { data } = await api.put<Binding>(`/bindings/${id}/notification`, {
      notification_enabled: enabled,
    });
    return data;
  },
};

// 导出默认实例
export default api;
