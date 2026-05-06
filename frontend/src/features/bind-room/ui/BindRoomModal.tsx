import { motion, AnimatePresence } from 'framer-motion';
import { X, Check, AlertCircle, ChevronRight, Home, KeyRound } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { SkeletonOptionCard } from '@/components/ui/SkeletonOptionCard';

import { useBindRoomModal } from '@/features/bind-room/model/useBindRoomModal';

interface BindRoomModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly onSuccess?: () => void;
}

export function BindRoomModal({ isOpen, onClose, onSuccess }: BindRoomModalProps) {
  const {
    currentStep,
    bindingProof,
    bindingProofRequired,
    clearError,
    error,
    finalRoom,
    handleBindingProofChange,
    handleBind,
    handleClose,
    handleGoBack,
    handleRestart,
    handleSelectOption,
    isLoading,
    isTouchDevice,
    options,
    queryError,
    refetch,
    selectedPath,
    stepLabels,
  } = useBindRoomModal({ isOpen, onClose, onSuccess });

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
            className="relative w-full max-w-[calc(100%-1rem)] sm:max-w-xl md:max-w-2xl lg:max-w-3xl max-h-[85dvh] sm:max-h-[90dvh] flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="relative p-4 sm:p-6 md:p-8 border-4 border-black shadow-[6px_6px_0_0_#000] sm:shadow-[8px_8px_0_0_#000] md:shadow-[10px_10px_0_0_#000] text-black bg-linear-to-br from-[#fff4c7] via-[#ffe173] to-[#ffc93c] flex-1 min-h-0 flex flex-col overflow-hidden">
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

              {/* 标题 - 响应式设计 */}
              <h3
                className="relative z-20 text-xl sm:text-2xl md:text-3xl font-black uppercase italic mb-4 sm:mb-6 text-black"
                style={{ textShadow: '2px 2px 0 #FACC15' }}
              >
                绑定房间
              </h3>

              {/* 步骤5预览（移到滚动容器外，避免 READY 标签被裁剪） */}
              {currentStep === 5 && finalRoom && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.95 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ duration: 0.3 }}
                  className="relative z-10 mb-6"
                >
                  <div className="relative overflow-visible border-4 border-black bg-linear-to-br from-yellow-200 via-yellow-100 to-white p-5 shadow-[6px_6px_0_0_#000] text-black">
                    <div className="absolute -top-3 -right-3 px-3 py-1 bg-brand-secondary border-2 border-black text-xs font-black uppercase tracking-[0.2em] shadow-[3px_3px_0_0_#000]">
                      READY!
                    </div>
                    <div className="flex items-center gap-3 mb-4">
                      <div className="w-10 h-10 bg-status-normal rounded-full border-2 border-black flex items-center justify-center shadow-[3px_3px_0_0_#000]">
                        <Check className="w-5 h-5 text-black" strokeWidth={3} />
                      </div>
                      <div>
                        <p className="text-xs font-black uppercase tracking-widest text-gray-600">Bingo!</p>
                        <span className="block text-lg sm:text-xl md:text-2xl font-black uppercase italic" style={{ textShadow: '2px 2px 0 #FACC15' }}>
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
                        <span className="px-2 py-1 bg-black text-white font-black text-xs uppercase tracking-widest shadow-[2px_2px_0_0_#000]">编号</span>
                        <span className="font-black text-base text-brand-primary">{finalRoom.roomid}</span>
                      </div>
                      {bindingProofRequired && (
                        <label className="block pt-2">
                          <span className="flex items-center gap-2 mb-2 text-xs font-black uppercase tracking-widest text-gray-700">
                            <KeyRound size={14} strokeWidth={3} />
                            绑定码
                          </span>
                          <input
                            value={bindingProof}
                            onChange={(event) => handleBindingProofChange(event.target.value)}
                            className="w-full border-2 border-black bg-white px-3 py-2 text-sm font-black uppercase tracking-widest outline-none shadow-[3px_3px_0_0_#000] focus:border-brand-primary"
                            placeholder="输入管理员提供的绑定码"
                            autoComplete="off"
                            spellCheck={false}
                          />
                        </label>
                      )}
                    </div>
                  </div>
                </motion.div>
              )}

              {/* 内容区域 - 响应式间距 */}
              <div className="relative z-10 space-y-4 sm:space-y-6 flex-1 overflow-y-auto pr-1">
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
                    {options.map((option) => {
                      // 使用完整路径作为 key，确保唯一性
                      // 避免不同层级间出现重复名称（如不同建筑都有"一楼"）
                      const fullPath = [...selectedPath, option.name].join('/');
                      
                      return (
                        <motion.button
                          key={fullPath}
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
                      );
                    })}
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
                            clearError();
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
                        disabled={isLoading || !finalRoom || (bindingProofRequired && !bindingProof.trim())}
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
