import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ArrowUpRight } from 'lucide-react';

interface CopyableQQNumberProps {
  readonly value: string;
}

export function CopyableQQNumber({ value }: CopyableQQNumberProps) {
  const [showToast, setShowToast] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setShowToast(true);
      setTimeout(() => setShowToast(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return (
    <span className="relative inline-block">
      <button
        type="button"
        onClick={handleCopy}
        className="inline-block bg-brand-secondary px-2 py-1 md:px-3 md:py-1.5 border border-black font-black text-black mx-1 md:mx-1.5 text-sm md:text-base lg:text-lg cursor-pointer hover:bg-yellow-400 active:scale-95 transition-all select-none"
        aria-label={`点击复制QQ号 ${value}`}
      >
        {value}
      </button>

      {/* 动态箭头指示器 */}
      <motion.div
        animate={{
          y: [0, -6, 0],
          x: [0, 2, 0],
        }}
        transition={{
          duration: 1.5,
          repeat: Infinity,
          ease: 'easeInOut',
        }}
        className="absolute -top-3 -right-3 md:-top-4 md:-right-4 pointer-events-none"
      >
        <ArrowUpRight 
          className="w-4 h-4 md:w-5 md:h-5 text-brand-primary drop-shadow-[0_2px_2px_rgba(0,0,0,0.3)]" 
          strokeWidth={3}
        />
      </motion.div>

      {/* 复制成功提示 */}
      <AnimatePresence>
        {showToast && (
          <motion.div
            initial={{ opacity: 0, y: 10, scale: 0.8 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -10, scale: 0.8 }}
            transition={{ duration: 0.2 }}
            className="absolute -top-10 left-1/2 -translate-x-1/2 whitespace-nowrap bg-black text-white px-3 py-1.5 rounded border-2 border-white shadow-[2px_2px_0_0_#000] text-xs md:text-sm font-bold z-50"
          >
            ✅ 已复制！
          </motion.div>
        )}
      </AnimatePresence>
    </span>
  );
}
