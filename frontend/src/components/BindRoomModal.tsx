import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Check, AlertCircle, ChevronRight, Home } from 'lucide-react';
import { roomApi, bindingApi } from '@/services/api';
import { Button } from '@/components/ui/Button';
import { SkeletonOptionCard } from '@/components/ui/SkeletonOptionCard';
import { usePathTree } from '@/hooks/usePathTree';
import type { PathChild, RoomByPathResponse } from '@/types';
import { useQueryClient } from '@tanstack/react-query';

interface BindRoomModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSuccess?: () => void;
}

export function BindRoomModal({ isOpen, onClose, onSuccess }: BindRoomModalProps) {
  const [currentStep, setCurrentStep] = useState(1); // 1=校区, 2=建筑, 3=楼层, 4=房间
  const [selectedPath, setSelectedPath] = useState<string[]>([]); // 已选路径
  const [error, setError] = useState<string | null>(null);
  const [finalRoom, setFinalRoom] = useState<RoomByPathResponse | null>(null);
  const [isTouchDevice, setIsTouchDevice] = useState(false);
  
  const queryClient = useQueryClient();
  const stepLabels = ['校区', '建筑', '楼层', '房间'];
  
  // 使用 React Query 查询当前路径的子节点
  const parent = selectedPath.join('/');
  const { 
    data: pathData, 
    isLoading, 
    error: queryError, // 用于 Task 4 错误显示
    refetch // 用于 Task 4 手动重试
  } = usePathTree(parent, isOpen && currentStep < 5);
  
  const options = pathData?.children || [];

  useEffect(() => {
    if (typeof globalThis !== 'undefined' && globalThis.window) {
      const mediaQuery = globalThis.window.matchMedia('(pointer: coarse), (hover: none)');
      const updateMatch = () => setIsTouchDevice(mediaQuery.matches);
      updateMatch();
      mediaQuery.addEventListener?.('change', updateMatch);
      return () => mediaQuery.removeEventListener?.('change', updateMatch);
    }
    return () => undefined;
  }, []);

  const reset = () => {
    setCurrentStep(1);
    setSelectedPath([]);
    setError(null);
    setFinalRoom(null);
  };

  const handleClose = () => {
    reset();
    onClose();
  };

  // React Query 自动处理加载和错误，无需手动 useEffect
  
  // 选择一个选项
  const handleSelectOption = async (option: PathChild) => {
    const newPath = [...selectedPath, option.name];
    setSelectedPath(newPath);
    setError(null);
    
    if (option.is_leaf) {
      // 叶子节点，查询房间详情
      try {
        const room = await roomApi.getRoomByPath(newPath.join('/'));
        setFinalRoom(room);
        setCurrentStep(5); // 进入确认步骤
      } catch {
        setError('查询房间失败，请稍后重试');
      }
    } else {
      // 非叶子节点，进入下一步
      setCurrentStep(currentStep + 1);
    }
  };
  
  // 返回上一步
  const handleGoBack = () => {
    if (currentStep > 1) {
      setSelectedPath(selectedPath.slice(0, -1));
      setCurrentStep(currentStep - 1);
      setError(null);
    }
  };
  
  // 重新开始（复用 reset 逻辑）
  const handleRestart = () => {
    reset();
  };

  // 确认绑定
  const handleBind = async () => {
    if (!finalRoom) return;

    setError(null);
    try {
      await bindingApi.createBinding(finalRoom.roomid);
      // 刷新房间列表
      await queryClient.invalidateQueries({ queryKey: ['rooms'] });
      await queryClient.invalidateQueries({ queryKey: ['flagged-rooms'] });
      await queryClient.invalidateQueries({ queryKey: ['bindings'] });
      
      onSuccess?.();
      handleClose();
    } catch {
      setError('绑定失败，请稍后重试或联系管理员');
    }
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          {/* 背景遮罩 */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="absolute inset-0 bg-black/90 backdrop-blur-sm"
            onClick={handleClose}
          />

          {/* 模态框内容 */}
          <motion.div
            initial={{ opacity: 0, scale: 0.5 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.5 }}
            transition={{ type: 'spring', damping: 15 }}
            className="relative w-full max-w-3xl max-h-[90vh]"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="relative overflow-hidden p-8 border-4 border-black shadow-[10px_10px_0_0_#000] text-black bg-linear-to-br from-[#fff4c7] via-[#ffe173] to-[#ffc93c] h-full flex flex-col">
              {/* 漫画半调纹理 */}
              <div
                className="absolute inset-0 opacity-15 pointer-events-none"
                style={{
                  backgroundImage: 'radial-gradient(circle, rgba(0,0,0,0.2) 8%, transparent 10%)',
                  backgroundSize: '18px 18px',
                }}
              />
              {/* 装饰元素 */}
              <div className="absolute -top-4 -right-4 w-12 h-12 bg-brand-secondary border-2 border-black z-30 shadow-[4px_4px_0_0_#000]" />
              <div className="absolute -bottom-4 -left-4 w-8 h-8 bg-brand-primary border-2 border-black z-30 shadow-[4px_4px_0_0_#000]" />

              {/* 关闭按钮 */}
              <button
                onClick={handleClose}
                className="absolute top-4 right-4 w-8 h-8 flex items-center justify-center bg-black text-white border-2 border-black hover:bg-brand-primary hover:scale-110 transition-all z-40"
              >
                <X size={16} strokeWidth={3} />
              </button>

              {/* 标题 */}
              <h3
                className="relative z-20 text-3xl font-black uppercase italic mb-6 text-black"
                style={{ textShadow: '3px 3px 0 #FACC15' }}
              >
                绑定房间
              </h3>

              {/* 内容区域 */}
              <div className="relative z-10 space-y-6 flex-1 overflow-y-auto pr-1">
                {/* 面包屑导航 */}
                {selectedPath.length > 0 && (
                  <div className="flex items-center gap-2 flex-wrap text-sm">
                    <button
                      onClick={handleRestart}
                      className="flex items-center gap-1 px-2 py-1 bg-blue-500 text-white border-2 border-black hover:bg-yellow-300 hover:text-black transition-colors font-bold"
                      title="重新开始"
                    >
                      <Home size={14} strokeWidth={3} />
                    </button>
                    {selectedPath.map((path, index) => (
                      <div key={`${index}-${path}`} className="flex items-center gap-2">
                        <ChevronRight size={16} strokeWidth={3} />
                        <span className="font-black text-black">{path}</span>
                      </div>
                    ))}
                  </div>
                )}

                {/* 当前步骤提示 */}
                {currentStep < 5 && (
                  <div className="text-center">
                    <div className="inline-block px-4 py-2 bg-brand-primary border-2 border-black shadow-[2px_2px_0_0_#000]">
                      <span className="font-black text-white text-lg uppercase italic">
                        {stepLabels[currentStep - 1]}
                      </span>
                    </div>
                  </div>
                )}

                {/* 选项网格（去除整体进场动画，避免内容在步骤切换时闪动） */}
                {currentStep < 5 && !isLoading && options.length > 0 && (
                  <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 max-h-80 overflow-y-auto">
                    {options.map((option) => (
                      <motion.button
                        key={option.name}
                        onClick={() => handleSelectOption(option)}
                        className="p-4 bg-white border-2 border-black hover:border-brand-primary hover:bg-brand-light transition-all duration-500 shadow-[4px_4px_0_0_#000] hover:shadow-[6px_6px_0_0_#000] text-left group"
                        whileHover={isTouchDevice ? undefined : {
                          y: -2,
                          transition: { duration: 0.5, delay: 0.1 }
                        }}
                        whileTap={isTouchDevice ? undefined : { scale: 0.95 }}
                      >
                        <div className="font-black text-base mb-1 text-black group-hover:text-brand-primary transition-colors">
                          {option.name}
                        </div>
                        {!option.is_leaf && (
                          <div className="text-xs text-gray-500 font-bold">
                            {option.room_count} 个房间
                          </div>
                        )}
                      </motion.button>
                    ))}
                  </div>
                )}

                {/* 加载状态 - 骨架屏 */}
                {isLoading && (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3"
                  >
                    {['a','b','c','d','e','f'].map((key) => (
                      <SkeletonOptionCard key={`skeleton-${key}`} />
                    ))}
                  </motion.div>
                )}

                {/* 最终预览（步骤5） */}
                <AnimatePresence>
                  {currentStep === 5 && finalRoom && (
                    <motion.div
                      initial={{ opacity: 0, scale: 0.95 }}
                      animate={{ opacity: 1, scale: 1 }}
                      exit={{ opacity: 0, scale: 0.95 }}
                    >
                      <div className="relative border-4 border-black bg-linear-to-br from-yellow-200 via-yellow-100 to-white p-5 shadow-[6px_6px_0_0_#000] text-black">
                        <div className="absolute -top-3 -right-3 px-3 py-1 bg-brand-secondary border-2 border-black text-xs font-black uppercase tracking-[0.2em] shadow-[3px_3px_0_0_#000]">
                          READY!
                        </div>
                        <div className="flex items-center gap-3 mb-4">
                          <div className="w-10 h-10 bg-status-normal rounded-full border-2 border-black flex items-center justify-center shadow-[3px_3px_0_0_#000]">
                            <Check className="w-5 h-5 text-black" strokeWidth={3} />
                          </div>
                          <div>
                            <p className="text-xs font-black uppercase tracking-widest text-gray-600">Bingo!</p>
                            <span className="block text-2xl font-black uppercase italic" style={{ textShadow: '2px 2px 0 #FACC15' }}>
                              找到房间!
                            </span>
                          </div>
                        </div>
                        <div className="space-y-3 text-sm border-t-2 border-dashed border-black/40 pt-3">
                          <div className="flex justify-between items-center">
                            <span className="px-2 py-1 bg-black text-white font-black text-xs uppercase tracking-widest shadow-[2px_2px_0_0_#000]">名称</span>
                            <span className="font-black text-base text-black">{finalRoom.room_name}</span>
                          </div>
                          <div className="flex justify-between items-center">
                            <span className="px-2 py-1 bg-black text-white font-black text-xs uppercase tracking-widest shadow-[2px_2px_0_0_#000]">位置</span>
                            <span className="font-black text-sm text-right max-w-[65%] text-black">{finalRoom.primary_roompath}</span>
                          </div>
                          <div className="flex justify-between items-center">
                            <span className="px-2 py-1 bg-black text-white font-black text-xs uppercase tracking-widest shadow-[2px_2px_0_0_#000]">剩余电量</span>
                            <span className="font-black text-base text-brand-primary">{finalRoom.electricity_fee.toFixed(2)} kWh</span>
                          </div>
                        </div>
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>

                {/* 错误提示 - 包含重试按钮 */}
                {(error || queryError) && (
                  <motion.div
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: 'auto' }}
                    className="bg-status-danger text-white border-2 border-black shadow-[2px_2px_0_0_#000] overflow-hidden"
                  >
                    <div className="p-4 space-y-3">
                      {/* 错误消息 */}
                      <div className="flex items-center gap-2 font-bold text-sm">
                        <AlertCircle size={16} strokeWidth={3} />
                        <span>{error || '加载失败，请重试'}</span>
                      </div>
                      
                      {/* React Query 错误详情（开发模式） */}
                      {queryError && import.meta.env.DEV && (
                        <div className="text-xs opacity-80 font-mono bg-black/20 p-2 border border-white/20">
                          {queryError instanceof Error ? queryError.message : String(queryError)}
                        </div>
                      )}
                      
                      {/* 重新加载按钮 */}
                      {queryError && (
                        <Button
                          onClick={() => {
                            setError(null);
                            refetch();
                          }}
                          variant="secondary"
                          size="sm"
                          fullWidth
                          className="bg-white text-black hover:bg-gray-200"
                        >
                          🔄 重新加载
                        </Button>
                      )}
                    </div>
                  </motion.div>
                )}

                {/* 操作按钮 */}
                <div className="flex gap-3">
                  {currentStep > 1 && currentStep < 5 && (
                    <Button
                      onClick={handleGoBack}
                      disabled={isLoading}
                      variant="secondary"
                      size="lg"
                      className="flex-1"
                    >
                      返回
                    </Button>
                  )}
                  {currentStep === 5 && (
                    <>
                      <Button
                        onClick={handleRestart}
                        disabled={isLoading}
                        variant="secondary"
                        size="lg"
                        className="flex-1"
                      >
                        重选
                      </Button>
                      <Button
                        onClick={handleBind}
                        disabled={isLoading || !finalRoom}
                        variant="primary"
                        size="lg"
                        isLoading={isLoading}
                        className="flex-1"
                      >
                        {isLoading ? '绑定中...' : '确认绑定'}
                      </Button>
                    </>
                  )}
                </div>
              </div>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}
