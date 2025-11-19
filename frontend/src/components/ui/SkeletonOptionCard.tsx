import { motion } from 'framer-motion';
import { ShimmerEffect } from './ShimmerEffect';

/**
 * 美漫风格骨架屏 - 模拟路径选项卡片
 * 用于加载状态下的占位显示
 */
export function SkeletonOptionCard() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      className="relative p-4 bg-white border-2 border-black overflow-hidden shadow-[4px_4px_0_0_#000]"
    >
      {/* Shimmer 滑动效果 */}
      <ShimmerEffect />

      {/* 标题占位符 - 脉动动画 */}
      <motion.div
        animate={{
          opacity: [0.5, 0.8, 0.5],
        }}
        transition={{
          duration: 1.5,
          repeat: Infinity,
          ease: 'easeInOut',
        }}
        className="h-5 w-3/4 bg-gray-300 mb-2 relative z-10"
      />

      {/* 副标题占位符（房间数量） */}
      <motion.div
        animate={{
          opacity: [0.4, 0.7, 0.4],
        }}
        transition={{
          duration: 1.5,
          repeat: Infinity,
          ease: 'easeInOut',
          delay: 0.2,
        }}
        className="h-3 w-1/2 bg-gray-200 relative z-10"
      />

      {/* 美漫风格半调纹理（装饰） */}
      <div className="absolute top-0 right-0 w-16 h-16 opacity-10 bg-[radial-gradient(#000_1px,transparent_1px)] bg-size-[4px_4px]" />
    </motion.div>
  );
}
