// 后端API响应类型定义

export type LoginMode = 'qq' | 'email';

export interface User {
  id: string;
  login_mode: LoginMode;
  identifier: string;
  qq_number?: string | null;
  email?: string | null;
  role: 'admin' | 'user';
  is_active: boolean;
}

export interface Room {
  id: string;
  roomid: string;
  electricity_fee: number;
  send_flag: boolean;
  threshold: number;
  room_name: string;
  primary_roompath: string;
  has_additional_paths: boolean;
  is_active: boolean;
  source_type: string;
  external_id?: string;
  last_synced_at?: string;
  created_at: string;
  updated_at: string;
}

export interface Binding {
  id: string;
  user_id: string;
  roomid: string;
  notification_enabled: boolean;
  created_at: string;
  updated_at: string;
  room?: Room;
}

export interface LoginResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

export interface ApiError {
  error: string;
  message: string;
}

// 路径树相关类型
export interface PathChild {
  name: string;
  is_leaf: boolean;
  room_count: number;
  roomid?: string;
}

export interface PathChildrenResponse {
  children: PathChild[];
  current_level: number;
  total_count: number;
}

export interface RoomByPathResponse {
  roomid: string;
  room_name: string;
  electricity_fee: number;
  threshold: number;
  primary_roompath: string;
}

// 前端状态类型
export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isSessionReady: boolean;
  login: (token: string, user: User) => void;
  logout: () => void;
  markSessionReady: () => void;
}

export interface DashboardStats {
  totalRooms: number;
  flaggedRooms: number;
  averageFee: number;
  todayConsumption: number;
}

// 房间状态枚举
export type RoomStatus = 'normal' | 'warning' | 'danger' | 'critical';

// 获取房间状态的辅助函数（剩余电量逻辑）
// electricity_fee: 剩余电量（kWh）
// threshold: 预警线（kWh）
// 逻辑：电量低于阈值时预警
export function getRoomStatus(room: Room): RoomStatus {
  const balance = room.electricity_fee;
  const warningLine = room.threshold;
  
  // 余额高于警戒线：正常
  if (balance >= warningLine) return 'normal';
  
  // 余额低于警戒线：根据差距判断严重程度
  const deficit = warningLine - balance; // 差额
  const deficitRatio = deficit / warningLine; // 差额占警戒线的比例
  
  if (deficitRatio >= 0.7) return 'critical'; // 差额≥70%警戒线（严重不足）
  if (deficitRatio >= 0.4) return 'danger';   // 差额≥40%警戒线（不足）
  if (deficitRatio > 0) return 'warning';     // 差额>0（刚低于警戒线）
  
  return 'normal';
}

// 格式化电量
export function formatElectricityFee(fee: number): string {
  return `${fee.toFixed(2)} kWh`;
}

// 格式化时间
export function formatTime(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  
  if (days === 0) return '今天';
  if (days === 1) return '昨天';
  if (days < 7) return `${days}天前`;
  
  return date.toLocaleDateString('zh-CN');
}
