import { motion } from 'framer-motion';

export function HeroDashboard() {
  return (
    <div className="relative py-8 px-4">
      {/* 漫画风格标题区域 */}
      <motion.div
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="text-center"
      >
        <div className="inline-block relative">
          <div className="absolute inset-0 bg-brand-secondary transform translate-x-3 translate-y-3 -skew-x-12 border-2 border-black" />
          <h1 className="hero-title relative bg-linear-to-br from-[#ffe173] to-[#ffd966] px-8 py-2 text-black border-4 border-black -skew-x-12 shadow-[6px_6px_0_0_#000] flex flex-col text-center">
            <span className="text-5xl font-black" style={{ textShadow: '2px 2px 0 #FACC15' }}>电力监控</span>
            <span className="text-4xl font-black" style={{ textShadow: '2px 2px 0 #FACC15' }}>系统</span>
          </h1>
        </div>
      </motion.div>
    </div>
  );
}

