import { motion } from 'framer-motion';

/**
 * 美漫风格 Shimmer 加载效果
 * 半透明光晕从左向右滑动，营造加载动画
 */
export function ShimmerEffect() {
  return (
    <motion.div
      initial={{ x: '-100%' }}
      animate={{ x: '200%' }}
      transition={{
        duration: 1.5,
        repeat: Infinity,
        repeatDelay: 0.5,
        ease: 'easeInOut',
      }}
      className="absolute inset-0 w-full h-full"
      style={{
        background:
          'linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent)',
      }}
    />
  );
}
