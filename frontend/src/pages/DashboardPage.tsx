import { lazy, Suspense } from 'react';
import { motion } from 'framer-motion';
import { Navbar } from '@/components/Navbar';
import { HeroDashboard } from '@/components/HeroDashboard';
import { RoomCard } from '@/components/RoomCard';
import { useDashboardPage } from '@/features/dashboard/model/useDashboardPage';
import { Button } from '@/components/ui/Button';

const AuthModal = lazy(() =>
  import('@/components/AuthModal').then((module) => ({ default: module.AuthModal })),
);
const InputModal = lazy(() =>
  import('@/components/ui/InputModal').then((module) => ({ default: module.InputModal })),
);
const RoomDetailModal = lazy(() =>
  import('@/components/RoomDetailModal').then((module) => ({ default: module.RoomDetailModal })),
);
const BindRoomModal = lazy(() =>
  import('@/features/bind-room').then((module) => ({ default: module.BindRoomModal })),
);

export default function DashboardPage() {
  const {
    bindings,
    getBindingForRoom,
    handleConfirmThreshold,
    handleDeleteBinding,
    handleEditThreshold,
    handleRoomClick,
    handleToggleNotification,
    isAuthModalOpen,
    isAuthenticated,
    isBindModalOpen,
    isDetailModalOpen,
    isLoading,
    isThresholdModalOpen,
    openAuthModal,
    openBindModal,
    rooms,
    selectedRoom,
    setIsAuthModalOpen,
    setIsBindModalOpen,
    setIsDetailModalOpen,
    setIsThresholdModalOpen,
    setSelectedRoom,
  } = useDashboardPage();

  // 渲染房间列表内容
  const renderRoomList = () => {
    if (!isAuthenticated) {
      return (
        <div className="col-span-full text-center py-20 bg-white/60 border-4 border-black border-dashed shadow-[6px_6px_0_0_#000]">
          <p className="text-gray-800 text-lg mb-4 font-bold">请登录以查看您的房间用电情况</p>
          <Button
            onClick={openAuthModal}
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

      return rooms.map((room, index) => (
        <RoomCard
          key={room.bindingId}
          room={room}
          binding={getBindingForRoom(room)}
          index={index}
          onClick={() => handleRoomClick(room)}
          onEditThreshold={handleEditThreshold}
          onToggleNotification={handleToggleNotification}
          onDeleteBinding={handleDeleteBinding}
        />
      ));
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
      <Navbar onLoginClick={openAuthModal} />

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
                  onClick={openBindModal}
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
      <Suspense fallback={null}>
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
      </Suspense>

      {/* 认证模态框 */}
      <Suspense fallback={null}>
        <AuthModal
          isOpen={isAuthModalOpen}
          onClose={() => setIsAuthModalOpen(false)}
          onSuccess={() => {
             setIsAuthModalOpen(false);
          }}
        />
      </Suspense>

      {/* 绑定模态框 */}
      <Suspense fallback={null}>
        <BindRoomModal
          isOpen={isBindModalOpen}
          onClose={() => setIsBindModalOpen(false)}
          onSuccess={() => {
             setIsBindModalOpen(false);
          }}
        />
      </Suspense>

      {/* 修改阈值模态框 */}
      <Suspense fallback={null}>
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
      </Suspense>
    </div>
  );
}
