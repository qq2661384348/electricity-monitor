import { motion } from 'framer-motion';
import { AlertCircle, TrendingUp, Zap } from 'lucide-react';
import { ComicModal } from '@/components/ui/comic-modal';
import type { Room, Binding, RoomStatus } from '@/types';
import { getRoomStatus, formatElectricityFee, formatTime } from '@/types';
import { cn } from '@/lib/utils';

interface RoomDetailModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly room: Room | null;
  readonly binding?: Binding;
}

const statusConfig: Record<
  RoomStatus,
  { label: string; badgeClass: string; icon: typeof AlertCircle }
> = {
  normal: {
    label: '正常',
    badgeClass:
      'bg-status-normal text-white border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: Zap,
  },
  warning: {
    label: '注意',
    badgeClass:
      'bg-status-warning text-black border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: TrendingUp,
  },
  danger: {
    label: '警告',
    badgeClass:
      'bg-status-danger text-white border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: AlertCircle,
  },
  critical: {
    label: '危险',
    badgeClass:
      'bg-status-critical text-white border-2 border-black shadow-[2px_2px_0_0_#000]',
    icon: AlertCircle,
  },
};

/**
 * RoomDetailModal - 使用 ComicModal 重构
 * 代码量从 230 行减少到 ~170 行（减少 26%）
 */
export function RoomDetailModal({ isOpen, onClose, room, binding }: RoomDetailModalProps) {
  if (!room) return null;

  const status = getRoomStatus(room);
  const config = statusConfig[status];
  const Icon = config.icon;
  const balance = room.electricity_fee;
  const warningLine = room.threshold;

  return (
    <ComicModal
      isOpen={isOpen}
      onClose={onClose}
      size="2xl"
      showCloseButton
      footer={
        <div className="flex justify-end w-full">
          <button
            onClick={onClose}
            className="px-6 py-2 bg-white text-black font-black text-sm uppercase border-2 border-black shadow-[3px_3px_0_0_#000] hover:shadow-[4px_4px_0_0_#000] hover:-translate-y-0.5 transition-all"
          >
            关闭
          </button>
        </div>
      }
    >
      <div className="space-y-4 sm:space-y-6">
        {/* 标题和状态 - 响应式设计 */}
        <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3 sm:gap-4">
                <div>
                  <h3
                    className="text-xl sm:text-2xl md:text-3xl font-black uppercase italic mb-2 text-black"
                    style={{ textShadow: '2px 2px 0 #FACC15' }}
                  >
                    {room.room_name}
                  </h3>
                  <p
                    className="text-sm font-black uppercase tracking-widest text-gray-900 bg-white/80 inline-block px-3 py-1 border-2 border-black shadow-[2px_2px_0_0_#000]"
                    style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}
                  >
                    {room.primary_roompath}
                  </p>
                </div>
                <div className="flex flex-row sm:flex-col items-center sm:items-end gap-2">
                  <div className={cn('flex items-center gap-2 px-3 py-1 transform -rotate-2', config.badgeClass)}>
                    <span className="text-sm font-black uppercase tracking-wider">
                      {config.label}
                    </span>
                    <Icon className="w-4 h-4" strokeWidth={3} />
                  </div>
                  <span className="text-xs font-mono text-gray-900 bg-white/80 px-2 py-0.5 border border-black/40 shadow-[1px_1px_0_0_#000]" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
                    ID: {room.roomid}
                  </span>
                </div>
        </div>

        {/* 主要信息区域 - 移动端单列 */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-6">
                <div className="space-y-4">
                  <div className="relative bg-white/70 p-4 pt-6 border-2 border-black shadow-[4px_4px_0_0_#000]">
                    <span className="absolute top-2 left-3 bg-[#ffd966] border-2 border-black px-2 text-sm text-black font-black uppercase shadow-[2px_2px_0_0_#000]">
                      当前剩余电量
                    </span>
                    <div className="flex items-baseline gap-1 mt-2">
                      <span
                        className="text-3xl sm:text-4xl md:text-5xl text-brand-primary tracking-tight"
                        style={{ fontFamily: '"Bangers", cursive', textShadow: '2px 2px 0 #FACC15' }}
                      >
                        {formatElectricityFee(room.electricity_fee)}
                      </span>
                    </div>
                  </div>

                  <div className="space-y-3 bg-white/70 p-4 border-2 border-black shadow-[4px_4px_0_0_#000]">
                    <div className="flex justify-between text-sm font-black text-gray-900" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
                      <span>预警线</span>
                      <span className="text-brand-primary">{warningLine.toFixed(2)} kWh</span>
                    </div>
                    <div className="flex justify-between text-sm font-black text-gray-900" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
                      <span>当前状态</span>
                      <span
                        className={cn(
                          status === 'normal' && 'text-status-normal',
                          status === 'warning' && 'text-status-warning',
                          status === 'danger' && 'text-status-danger',
                          status === 'critical' && 'text-status-critical'
                        )}
                      >
                        {balance >= warningLine ? '电量充足' : `剩余 ${balance.toFixed(2)} kWh`}
                      </span>
                    </div>
                    <div className="relative h-4 bg-gray-900 border-2 border-black">
                      <motion.div
                        initial={{ width: 0 }}
                        animate={{
                          width:
                            balance >= warningLine
                              ? '100%'
                              : `${Math.max((balance / warningLine) * 100, 5)}%`,
                        }}
                        transition={{ duration: 1, ease: 'easeOut' }}
                        className={cn(
                          'h-full border-r-2 border-black relative overflow-hidden',
                          status === 'normal' && 'bg-status-normal',
                          status === 'warning' && 'bg-status-warning',
                          status === 'danger' && 'bg-status-danger',
                          status === 'critical' && 'bg-status-critical'
                        )}
                      >
                        <div className="absolute inset-0 bg-[linear-gradient(45deg,rgba(0,0,0,0.2)_25%,transparent_25%,transparent_50%,rgba(0,0,0,0.2)_50%,rgba(0,0,0,0.2)_75%,transparent_75%,transparent)] bg-size-[8px_8px]" />
                      </motion.div>
                    </div>
                  </div>
                </div>

                <div className="space-y-4 text-sm">
                  <div className="bg-white/70 p-4 border-2 border-black shadow-[4px_4px_0_0_#000] space-y-2">
                    <div className="flex justify-between items-center">
                      <span className="px-2 py-1 bg-black text-white font-black text-xs uppercase tracking-widest shadow-[2px_2px_0_0_#000]">
                        绑定信息
                      </span>
                      <span className="text-xs font-mono text-gray-900" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
                        最后更新 {formatTime(room.updated_at)}
                      </span>
                    </div>
                    <div className="grid grid-cols-2 gap-2 mt-2 text-sm font-black text-gray-900" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
                      <div>绑定ID</div>
                      <div className="font-mono break-all">
                        {binding?.id ?? '未提供'}
                      </div>
                      <div>通知状态</div>
                      <div>
                        {binding?.notification_enabled ? '🔔 已开启通知' : '🔕 未开启通知'}
                      </div>
                    </div>
                  </div>

                  <div className="bg-white/70 p-4 border-2 border-black shadow-[4px_4px_0_0_#000] space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="px-2 py-1 bg-black text-white font-black text-xs uppercase tracking-widest shadow-[2px_2px_0_0_#000]">
                        说明
                      </span>
                    </div>
                    <p className="text-sm text-gray-900 leading-relaxed font-black" style={{ textShadow: '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff' }}>
                      当电量低于预警线时，房间状态会从“正常”逐步变为“注意”、“警告”或“危险”，请及时补充以避免停电。
                    </p>
                  </div>
                </div>
        </div>
      </div>
    </ComicModal>
  );
}
