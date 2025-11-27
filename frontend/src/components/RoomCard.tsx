import { useState } from 'react';
import { motion } from 'framer-motion';
import { Zap, AlertCircle, TrendingUp } from 'lucide-react';
import type { Room, RoomStatus, Binding } from '@/types';
import { getRoomStatus, formatElectricityFee, formatTime } from '@/types';
import { cn } from '@/lib/utils';
import { NotificationModal } from './ui/NotificationModal';
import { ConfirmModal } from './ui/ConfirmModal';

interface RoomCardProps {
  readonly room: Room & { bindingId?: string }; // 扩展Room类型，注入bindingId
  readonly binding?: Binding; // 绑定信息（包含notification_enabled）
  readonly onClick?: () => void;
  readonly onEditThreshold?: (room: Room) => void; // 修改阈值回调
  readonly onToggleNotification?: (bindingId: string, enabled: boolean) => Promise<void>; // 切换通知回调
  readonly onDeleteBinding?: (bindingId: string) => Promise<void>; // 删除绑定回调
  readonly index?: number;
}

const statusConfig: Record<RoomStatus, { color: string; bg: string; icon: typeof AlertCircle; label: string }> = {
  normal: {
    color: 'text-white',
    bg: 'bg-status-normal border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: Zap,
    label: '正常',
  },
  warning: {
    color: 'text-black',
    bg: 'bg-status-warning border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: TrendingUp,
    label: '注意',
  },
  danger: {
    color: 'text-white',
    bg: 'bg-status-danger border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: AlertCircle,
    label: '警告',
  },
  critical: {
    color: 'text-white',
    bg: 'bg-status-critical border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: AlertCircle,
    label: '危险',
  },
};

export function RoomCard({ 
  room, 
  binding,
  onClick, 
  onEditThreshold, 
  onToggleNotification,
  onDeleteBinding, 
  index = 0 
}: RoomCardProps) {
  const [isNotificationModalOpen, setIsNotificationModalOpen] = useState(false);
  const [isDeleteModalOpen, setIsDeleteModalOpen] = useState(false);
  const status = getRoomStatus(room);
  const config = statusConfig[status];
  const Icon = config.icon;
  
  // 剩余电量逻辑：显示电量相对于阈值的情况
  const balance = room.electricity_fee;
  const warningLine = room.threshold;

  return (
    <motion.div
      initial={{ opacity: 0, y: 50 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.05, duration: 0.4, type: 'spring' }}
      onClick={onClick}
      className="group cursor-pointer"
    >
      <div className={cn(
        "relative overflow-hidden p-5 h-full",
        "bg-linear-to-br from-[#fff8dc] via-[#ffe9a3] to-[#ffd966]",
        "border-4 border-black shadow-[6px_6px_0_0_#000]",
        "transition-all duration-200 group-hover:shadow-[8px_8px_0_0_#000] group-hover:border-brand-primary",
        "group-focus-visible:outline-2 group-focus-visible:outline-brand-secondary"
      )}>
        {/* 漫画半调纹理装饰 */}
        <div
          className="absolute inset-0 opacity-10 pointer-events-none"
          style={{
            backgroundImage: 'radial-gradient(circle, rgba(0,0,0,0.15) 8%, transparent 10%)',
            backgroundSize: '16px 16px',
          }}
        />
        
        {/* 状态标签 (类似漫画贴纸) */}
        <div className="flex items-center justify-between mb-4 relative z-10">
          <div className={cn("flex items-center gap-2 px-3 py-1 transform -rotate-2", config.bg)}>
            <span className={cn("text-sm font-black uppercase tracking-wider", config.color)}>
              {config.label}
            </span>
          </div>
          <div className="bg-black p-1.5 border-2 border-white shadow-[2px_2px_0_0_#fff]">
            <Icon className={cn("w-5 h-5 text-white")} strokeWidth={3} />
          </div>
        </div>

        {/* 房间信息 */}
        <h3 className="relative z-10 text-2xl text-black mb-1 group-hover:text-brand-primary transition-colors tracking-wide" style={{ fontFamily: '"Bangers", cursive', textShadow: '2px 2px 0 #FACC15' }}>
          {room.room_name}
        </h3>
        <p
          className="relative z-10 text-gray-900 text-sm font-black uppercase tracking-[0.3em] mb-6 border-b-2 border-black/30 pb-2 inline-block"
          style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}
        >
          {room.primary_roompath}
        </p>

        {/* 电量信息 */}
        <div className="space-y-4">
          {/* 当前电量 */}
          <div className="relative z-10 bg-white/50 p-3 border-2 border-black rounded-none shadow-[3px_3px_0_0_#000]">
            <span className="absolute -top-3 left-2 bg-[#ffd966] border-2 border-black px-2 text-xs text-black font-black uppercase shadow-[2px_2px_0_0_#000]">当前剩余电量</span>
            <div className="flex items-baseline gap-1">
              <span className="text-4xl text-brand-primary tracking-tight" style={{ fontFamily: '"Bangers", cursive', textShadow: '2px 2px 0 #FACC15' }}>
                {formatElectricityFee(room.electricity_fee)}
              </span>
            </div>
          </div>

          {/* 电量状态 - 漫画风格 */}
          <div className="relative z-10 space-y-2">
            {/* 阈值信息 */}
            <div className="flex justify-between text-sm font-black tracking-wide" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
              <span className="text-gray-800">预警线</span>
              <span className="text-brand-primary">{warningLine.toFixed(2)} kWh</span>
            </div>
            
            {/* 电量状态 */}
            <div className="flex justify-between text-sm font-black tracking-wide" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
              <span className="text-gray-800">状态</span>
              <span
                className={cn(
                  status === 'normal' && "text-status-normal",
                  status === 'warning' && "text-status-warning",
                  status === 'danger' && "text-status-danger",
                  status === 'critical' && "text-status-critical"
                )}
              >
                {balance >= warningLine ? '电量充足' : `剩余 ${balance.toFixed(2)} kWh`}
              </span>
            </div>
            
            {/* 视觉进度条（相对于阈值） */}
            <div className="relative h-4 bg-gray-900 border-2 border-black">
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: balance >= warningLine ? '100%' : `${Math.max((balance / warningLine) * 100, 5)}%` }}
                transition={{ duration: 1, ease: 'easeOut' }}
                className={cn(
                  "h-full border-r-2 border-black relative overflow-hidden",
                  status === 'normal' && "bg-status-normal",
                  status === 'warning' && "bg-status-warning",
                  status === 'danger' && "bg-status-danger",
                  status === 'critical' && "bg-status-critical"
                )}
              >
                {/* 进度条纹理 */}
                <div className="absolute inset-0 bg-[linear-gradient(45deg,rgba(0,0,0,0.2)_25%,transparent_25%,transparent_50%,rgba(0,0,0,0.2)_50%,rgba(0,0,0,0.2)_75%,transparent_75%,transparent)] bg-size-[8px_8px]" />
              </motion.div>
            </div>
          </div>

          {/* 底部信息 */}
          <div className="relative z-10 flex items-center justify-between text-sm text-gray-900 font-mono font-black mb-3" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
            <span>ID: {room.roomid}</span>
            <span>{formatTime(room.updated_at)}</span>
          </div>
          
          {/* 操作按钮组 - 移动端横向布局 */}
          {(onEditThreshold || onToggleNotification || onDeleteBinding) && (
            <div className="relative z-10 flex flex-wrap gap-2 pt-3 border-t-2 border-black/30">
              {onEditThreshold && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onEditThreshold(room);
                  }}
                  className="flex-1 min-w-[80px] px-2 sm:px-3 py-2 bg-brand-secondary text-black font-black text-sm sm:text-base uppercase border-2 border-black hover:scale-105 transition-all shadow-[2px_2px_0_0_#000] hover:shadow-[3px_3px_0_0_#000]"
                  style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}
                >
                  <span className="flex items-center justify-center gap-1 sm:gap-2">
                    <span className="text-lg sm:text-xl" aria-hidden>✏️</span>
                    <span className="hidden xs:inline sm:inline">修改阈值</span>
                    <span className="xs:hidden sm:hidden">阈值</span>
                  </span>
                </button>
              )}
              {onToggleNotification && room.bindingId && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsNotificationModalOpen(true);
                  }}
                  className={`flex-1 min-w-[80px] px-2 sm:px-3 py-2 font-black text-sm uppercase border-2 border-black hover:scale-105 transition-all shadow-[2px_2px_0_0_#000] hover:shadow-[3px_3px_0_0_#000] ${
                    binding?.notification_enabled
                      ? 'bg-status-normal text-white'
                      : 'bg-gray-400 text-black'
                  }`}
                  style={{ textShadow: binding?.notification_enabled ? '-1px 0 #000, 1px 0 #000, 0 1px #000, 0 -1px #000' : '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}
                >
                  <span className="flex items-center justify-center gap-1 sm:gap-2">
                    <span className="text-lg sm:text-xl" aria-hidden>
                      {binding?.notification_enabled ? '🔔' : '🔕'}
                    </span>
                    <span className="hidden sm:inline">{binding?.notification_enabled ? '已开启' : '已关闭'}</span>
                  </span>
                </button>
              )}
              {onDeleteBinding && room.bindingId && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setIsDeleteModalOpen(true);
                  }}
                  className="flex-1 min-w-[80px] px-2 sm:px-3 py-2 bg-status-danger text-white font-black text-sm uppercase border-2 border-black hover:scale-105 transition-all shadow-[2px_2px_0_0_#000] hover:shadow-[3px_3px_0_0_#000]"
                  style={{ textShadow: '-1px 0 #000, 1px 0 #000, 0 1px #000, 0 -1px #000' }}
                >
                  <span className="flex items-center justify-center gap-1 sm:gap-2">
                    <span className="text-lg sm:text-xl" aria-hidden>🗑️</span>
                    <span className="hidden sm:inline">删除</span>
                  </span>
                </button>
              )}
            </div>
          )}
        </div>
      </div>
      
      {/* 悬浮光晕 */}
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none">
        <div className="absolute inset-0 bg-linear-to-br from-brand-secondary/10 to-transparent rounded-none" />
      </div>

      {/* 通知模态框 */}
      {binding && onToggleNotification && (
        <NotificationModal
          isOpen={isNotificationModalOpen}
          onClose={() => setIsNotificationModalOpen(false)}
          onConfirm={async (enabled) => {
            if (room.bindingId) {
              await onToggleNotification(room.bindingId, enabled);
            }
          }}
          currentStatus={binding.notification_enabled}
          roomName={room.room_name}
        />
      )}

      {/* 删除确认模态框 */}
      {onDeleteBinding && room.bindingId && (
        <ConfirmModal
          isOpen={isDeleteModalOpen}
          onClose={() => setIsDeleteModalOpen(false)}
          onConfirm={async () => {
            if (room.bindingId) {
              await onDeleteBinding(room.bindingId);
            }
          }}
          title="确认删除"
          message={`确定要删除房间 "${room.room_name}" 的绑定吗？删除后将无法查看该房间的电费信息。`}
          confirmText="删除"
          cancelText="取消"
          isDestructive
        />
      )}
    </motion.div>
  );
}
