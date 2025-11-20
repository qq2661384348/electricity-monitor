import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { BookOpen, UserPlus, LogIn, Link2, Settings, Bell, ChevronDown, ChevronUp } from 'lucide-react';

interface TutorialStep {
  id: number;
  icon: React.ReactNode;
  title: string;
  description: string;
  image?: string;
  highlight?: string;
}

const tutorialSteps: TutorialStep[] = [
  {
    id: 1,
    icon: <UserPlus className="w-6 h-6" strokeWidth={3} />,
    title: '添加QQ好友',
    description: '先添加已经部署并登录的 NapCat 机器人账号为好友；具体账号由部署者在私有渠道提供。',
    highlight: '100000002',
  },
  {
    id: 2,
    icon: <LogIn className="w-6 h-6" strokeWidth={3} />,
    title: '登录验证',
    description: '使用接收通知的 QQ 号登录，验证码会通过机器人私聊发送。',
    image: 'https://cdn4.winhlb.com/2025/11/20/691f2b16cde0e.png',
  },
  {
    id: 3,
    icon: <Link2 className="w-6 h-6" strokeWidth={3} />,
    title: '绑定房间',
    description: '点击“绑定房间”按钮，按照步骤选择你的房间位置',
  },
  {
    id: 4,
    icon: <BookOpen className="w-6 h-6" strokeWidth={3} />,
    title: '查看绑定',
    description: '绑定成功后可以看到房间信息和当前电量',
    image: 'https://cdn4.winhlb.com/2025/11/20/691f2b1c8c846.png',
  },
  {
    id: 5,
    icon: <Settings className="w-6 h-6" strokeWidth={3} />,
    title: '设置阈值',
    description: '开启通知并设定阈值（如100kWh），低于阈值时会通知',
    image: 'https://cdn4.winhlb.com/2025/11/20/691f2b1c42564.png',
    highlight: '100kWh',
  },
  {
    id: 6,
    icon: <Bell className="w-6 h-6" strokeWidth={3} />,
    title: '接收通知',
    description: '电量低于阈值时，系统会自动发送机器人通知。',
    image: 'https://cdn4.winhlb.com/2025/11/20/691f2b178a2df.png',
  },
];

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      type: 'spring' as const,
      stiffness: 100,
    },
  },
};

export function HeroDashboard() {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <motion.div
      initial={{ opacity: 0, y: -20 }}
      animate={{ opacity: 1, y: 0 }}
      className="relative py-8 px-4"
    >
      {/* 标题区域 */}
      <div className="text-center mb-6">
        <div className="inline-block relative mb-4">
          <div className="absolute inset-0 bg-brand-secondary transform translate-x-2 translate-y-2 border-2 border-black" />
          <h2 className="relative bg-linear-to-br from-[#ffe173] to-[#ffd966] px-4 py-2 md:px-6 md:py-3 text-black border-4 border-black shadow-[4px_4px_0_0_#000] font-black text-xl md:text-2xl lg:text-3xl">
            🎓 面向校园宿舍场景的电费提醒系统
          </h2>
        </div>
        <div className="inline-block bg-white border-2 border-black px-3 py-1.5 md:px-4 md:py-2 shadow-[2px_2px_0_0_#000]">
          <p className="font-bold text-xs md:text-sm lg:text-base">
            📱 使用方法（仅支持QQ好友通知）
          </p>
        </div>
      </div>

      {/* 折叠/展开按钮 */}
      <div className="text-center mb-6">
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="inline-flex items-center gap-2 px-6 py-3 bg-brand-primary text-black border-4 border-black shadow-[4px_4px_0_0_#000] hover:shadow-[6px_6px_0_0_#000] hover:-translate-y-1 active:translate-y-0 active:shadow-[2px_2px_0_0_#000] transition-all font-black text-base md:text-lg uppercase"
        >
          <BookOpen className="w-5 h-5 md:w-6 md:h-6" strokeWidth={3} />
          <span>{isExpanded ? '收起教程' : '查看使用教程'}</span>
          {isExpanded ? (
            <ChevronUp className="w-5 h-5 md:w-6 md:h-6" strokeWidth={3} />
          ) : (
            <ChevronDown className="w-5 h-5 md:w-6 md:h-6" strokeWidth={3} />
          )}
          {/* 半调纹理 */}
          <div className="absolute inset-0 bg-[radial-gradient(rgba(0,0,0,0.2)_1px,transparent_1px)] bg-size-[4px_4px] opacity-30 pointer-events-none" />
        </button>
      </div>

      {/* 教程步骤网格（手风琴式） */}
      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.3, ease: 'easeInOut' }}
            className="overflow-hidden"
          >
            <motion.div
              variants={containerVariants}
              initial="hidden"
              animate="visible"
              className="max-w-7xl mx-auto grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 md:gap-6 pt-4"
            >
              {tutorialSteps.map((step) => (
                <motion.div
                  key={step.id}
                  variants={itemVariants}
                  whileHover={{ y: -4, transition: { duration: 0.2 } }}
                  className="relative bg-white border-4 border-black shadow-[4px_4px_0_0_#000] hover:shadow-[6px_6px_0_0_#000] transition-all p-4 md:p-6 group"
                >
                  {/* 半调纹理 */}
                  <div className="absolute inset-0 bg-[radial-gradient(rgba(0,0,0,0.1)_1px,transparent_1px)] bg-size-[4px_4px] opacity-20 pointer-events-none" />
                  
                  {/* 步骤编号徽章 */}
                  <div className="absolute -top-2 -left-2 md:-top-3 md:-left-3 w-10 h-10 md:w-12 md:h-12 bg-brand-primary border-2 border-black shadow-[3px_3px_0_0_#000] flex items-center justify-center z-10">
                    <span className="font-black text-xl md:text-2xl text-black">{step.id}</span>
                  </div>

                  {/* 图标 */}
                  <div className="mb-3 md:mb-4 flex items-center gap-2 md:gap-3">
                    <div className="w-8 h-8 md:w-10 md:h-10 bg-brand-secondary border-2 border-black flex items-center justify-center shadow-[2px_2px_0_0_#000]">
                      {step.icon}
                    </div>
                    <h3 className="font-black text-lg md:text-xl text-black">{step.title}</h3>
                  </div>

                  {/* 描述 */}
                  <p className="text-xs md:text-sm text-gray-700 font-bold mb-3 md:mb-4 leading-relaxed">
                    {step.description.split(step.highlight || '').map((part, i, arr) => (
                      <span key={`${step.id}-desc-${i}`}>
                        {part}
                        {i < arr.length - 1 && step.highlight && (
                          <span className="inline-block bg-brand-secondary px-1.5 py-0.5 md:px-2 border border-black font-black text-black mx-0.5 md:mx-1 text-xs md:text-sm">
                            {step.highlight}
                          </span>
                        )}
                      </span>
                    ))}
                  </p>

                  {/* 图片 */}
                  {step.image && (
                    <div className="relative border-2 border-black overflow-hidden shadow-[2px_2px_0_0_#000] bg-gray-100">
                      <img
                        src={step.image}
                        alt={`步骤${step.id}示例`}
                        className="w-full h-auto object-cover"
                        loading="lazy"
                        onError={(e) => {
                          const target = e.target as HTMLImageElement;
                          target.style.display = 'none';
                          const placeholder = target.nextElementSibling as HTMLElement;
                          if (placeholder) placeholder.style.display = 'flex';
                        }}
                      />
                      <div
                        className="hidden w-full h-24 md:h-32 items-center justify-center bg-gray-200 text-gray-500 font-bold text-xs md:text-sm"
                        style={{ display: 'none' }}
                      >
                        🖼️ 图片加载失败
                      </div>
                    </div>
                  )}
                </motion.div>
              ))}
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

