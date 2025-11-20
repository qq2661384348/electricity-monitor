import { useState } from 'react';
import { motion } from 'framer-motion';
import { Navbar } from '@/components/Navbar';
import { AuthModal } from '@/components/AuthModal';
import { BindRoomModal } from '@/components/BindRoomModal';
import { InputModal } from '@/components/ui/InputModal';
import { HeroDashboard } from '@/components/HeroDashboard';
import { RoomCard } from '@/components/RoomCard';
import { RoomDetailModal } from '@/components/RoomDetailModal';
import { useAuthStore } from '@/stores/authStore';
import { useBindingsQuery } from '@/features/dashboard/hooks/useDashboardData';
import { Button } from '@/components/ui/Button';
import { roomApi, bindingApi } from '@/services/api';
import { useQueryClient } from '@tanstack/react-query';

import type { Room } from '@/types';

export default function DashboardPage() {
  const [isAuthModalOpen, setIsAuthModalOpen] = useState(false);
  const [isBindModalOpen, setIsBindModalOpen] = useState(false);
  const [isThresholdModalOpen, setIsThresholdModalOpen] = useState(false);
  const [selectedRoom, setSelectedRoom] = useState<(Room & { bindingId?: string }) | null>(null);
  const [isDetailModalOpen, setIsDetailModalOpen] = useState(false);
  const { isAuthenticated } = useAuthStore();
  const queryClient = useQueryClient();

  // 使用 React Query 获取数据（Binding 为核心数据源）
  const { data: bindings = [], isLoading: isLoadingBindings } = useBindingsQuery();
  
  const isLoading = isLoadingBindings;
  
  // 从绑定中提取房间信息（注入 bindingId 供删除使用）
  const rooms = bindings
    .filter((b) => b.room) // 过滤无效绑定
    .map((b) => ({
      ...b.room!,
      bindingId: b.id, // 注入 bindingId
    }));

  // 处理需要登录的操作
  const requireAuth = (action: () => void) => {
    if (isAuthenticated) {
      action();
    } else {
      setIsAuthModalOpen(true);
    }
  };

  const handleRoomClick = (room: Room & { bindingId?: string }) => {
    requireAuth(() => {
      setSelectedRoom(room);
      setIsDetailModalOpen(true);
    });
  };

  // 修改阈值
  const handleEditThreshold = (room: Room) => {
    setSelectedRoom(room);
    setIsThresholdModalOpen(true);
  };

  // 确认修改阈值
  const handleConfirmThreshold = async (value: string) => {
    if (!selectedRoom) return;
    
    await roomApi.updateThreshold(selectedRoom.id, Number(value));
    await queryClient.invalidateQueries({ queryKey: ['bindings'] });
  };

  // 切换通知
  const handleToggleNotification = async (bindingId: string, enabled: boolean) => {
    await bindingApi.updateNotificationEnabled(bindingId, enabled);
    await queryClient.invalidateQueries({ queryKey: ['bindings'] });
  };

  // 删除绑定（不需要confirm，由ConfirmModal处理）
  const handleDeleteBinding = async (bindingId: string) => {
    await bindingApi.deleteBinding(bindingId);
    await queryClient.invalidateQueries({ queryKey: ['bindings'] });
  };

  // 渲染房间列表内容
  const renderRoomList = () => {
    if (!isAuthenticated) {
      return (
        <div className="col-span-full text-center py-20 bg-white/60 border-4 border-black border-dashed shadow-[6px_6px_0_0_#000]">
          <p className="text-gray-800 text-lg mb-4 font-bold">请登录以查看您的房间用电情况</p>
          <Button
            onClick={() => setIsAuthModalOpen(true)}
            size="lg"
            variant="primary"
          >
            立即登录
          </Button>
        </div>
      );
    }

    if (rooms.length === 0 && !isLoading) {
      return (
        <div className="col-span-full text-center py-20 bg-white/60 border-4 border-black border-dashed shadow-[6px_6px_0_0_#000]">
           <p className="text-gray-800 text-lg mb-2 font-bold">暂无绑定的房间</p>
           <p className="text-gray-600 text-sm">请点击标题栏的 "绑定房间" 按钮添加房间</p>
        </div>
      );
    }

    return rooms.map((room, index) => {
      // 找到对应的 binding
      const binding = bindings.find((b) => b.id === room.bindingId);
      
      return (
        <RoomCard
          key={room.bindingId}
          room={room}
          binding={binding}
          index={index}
          onClick={() => handleRoomClick(room)}
          onEditThreshold={handleEditThreshold}
          onToggleNotification={handleToggleNotification}
          onDeleteBinding={handleDeleteBinding}
        />
      );
    });
  };

  return (
    <div className="relative min-h-screen bg-[#fef9e7] overflow-hidden">
      {/* 美漫浅色渐变背景 */}
      <div className="absolute inset-0 -z-10 bg-linear-to-br from-[#fff9e6] via-[#fef5d4] to-[#ffeaa7]" />
      {/* 彩色半调纹理 */}
      <div
        className="absolute inset-0 opacity-[0.15] -z-10"
        style={{
          backgroundImage: 'radial-gradient(circle, rgba(14,165,233,0.3) 8%, transparent 10%), radial-gradient(circle, rgba(250,204,21,0.2) 5%, transparent 7%)',
          backgroundSize: '40px 40px, 25px 25px',
          backgroundPosition: '0 0, 20px 20px',
        }}
      />
      {/* 装饰性色块 */}
      <div className="absolute top-20 right-10 w-64 h-64 bg-brand-secondary/10 border-4 border-black/20 transform rotate-12 -z-10" />
      <div className="absolute bottom-40 left-20 w-48 h-48 bg-brand-primary/10 border-4 border-black/20 transform -rotate-6 -z-10" />

      {/* 导航栏 */}
      <Navbar onLoginClick={() => setIsAuthModalOpen(true)} />

      {/* 主内容 */}
      <main className="relative z-10 pt-20">
        {/* 英雄仪表盘 (纯标题 Banner) */}
        <HeroDashboard />

        {/* 我的房间列表 */}
        <section className="max-w-6xl mx-auto px-4 py-12">
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            className="flex flex-col md:flex-row gap-4 md:gap-6 md:items-center md:justify-between mb-8 text-center md:text-left"
          >
            <h2 className="text-3xl font-bold text-black flex flex-col sm:flex-row items-center gap-2 md:gap-3">
              <span className="text-brand-primary text-4xl sm:text-3xl">⚡</span>
              <span style={{ fontFamily: '"Bangers", cursive', textShadow: '2px 2px 0 #FACC15' }}>我的房间</span>
              {!isAuthenticated && (
                <span className="mt-2 sm:mt-0 sm:ml-3 mb-0 text-xs px-3 py-1 bg-white border-2 border-black shadow-[2px_2px_0_0_#000] font-bold">
                  登录后绑定查看相关数据
                </span>
              )}
            </h2>
            
            {/* 绑定管理按钮组 */}
            {isAuthenticated && (
              <div className="flex flex-col sm:flex-row items-center gap-3 w-full md:w-auto">
                {/* 房间数量徽章 */}
                <div className="w-full sm:w-auto px-4 py-2 md:px-6 md:py-3 lg:px-8 lg:py-4 bg-white border-2 border-black shadow-[2px_2px_0_0_#000]">
                  <span className="font-black text-brand-primary text-sm md:text-base lg:text-lg uppercase">
                    {bindings.length} 个房间
                  </span>
                </div>
                
                {/* 绑定房间按钮 */}
                <Button
                  onClick={() => setIsBindModalOpen(true)}
                  variant="primary"
                  size="lg"
                  className="flex items-center gap-2 w-full sm:w-auto md:text-base! lg:text-lg!"
                >
                  <span className="text-xl md:text-2xl">+</span>
                  <span>绑定房间</span>
                </Button>
              </div>
            )}
          </motion.div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
             {renderRoomList()}
          </div>
        </section>

        {/* Footer */}
        <footer className="max-w-6xl mx-auto px-4 py-8 text-center text-gray-700 text-sm">
          <p className="font-bold">由广西科技大学 计算机科学与技术学院(软件学院) 物联网工程231班 赖永杰制作</p>
        </footer>
      </main>

      {/* 房间详情模态框 */}
      <RoomDetailModal
        isOpen={isDetailModalOpen}
        onClose={() => {
          setIsDetailModalOpen(false);
          setSelectedRoom(null);
        }}
        room={selectedRoom}
        binding={
          selectedRoom?.bindingId
            ? bindings.find((b) => b.id === selectedRoom.bindingId)
            : undefined
        }
      />

      {/* 认证模态框 */}
      <AuthModal
        isOpen={isAuthModalOpen}
        onClose={() => setIsAuthModalOpen(false)}
        onSuccess={() => {
           setIsAuthModalOpen(false);
        }}
      />

      {/* 绑定模态框 */}
      <BindRoomModal
        isOpen={isBindModalOpen}
        onClose={() => setIsBindModalOpen(false)}
        onSuccess={() => {
           setIsBindModalOpen(false);
        }}
      />

      {/* 修改阈值模态框 */}
      <InputModal
        isOpen={isThresholdModalOpen}
        onClose={() => setIsThresholdModalOpen(false)}
        onConfirm={handleConfirmThreshold}
        title="修改电量阈值"
        label="新阈值（kWh）"
        placeholder="请输入新的预警阈值（kWh）"
        defaultValue={selectedRoom?.threshold.toString()}
        inputType="number"
        helpText="当剩余电量低于此值时将触发预警"
        validator={(value) => {
          const num = Number(value);
          if (Number.isNaN(num) || num <= 0) {
            return '请输入有效的正数';
          }
          if (num > 10000) {
            return '阈值不能超过10000 kWh';
          }
          return null;
        }}
      />
    </div>
  );
}
