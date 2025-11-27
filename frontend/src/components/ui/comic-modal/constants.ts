import type { ModalSize, DecorationConfig } from './types';

/**
 * 模态框尺寸映射到 Tailwind CSS 类名
 * 
 * 响应式设计：
 * - 移动端：使用 calc(100% - 1rem) 确保边距
 * - 桌面端：使用固定最大宽度
 */
export const MODAL_SIZE_CLASSES: Record<ModalSize, string> = {
  sm: 'max-w-[calc(100%-1rem)] sm:max-w-sm',
  md: 'max-w-[calc(100%-1rem)] sm:max-w-md',
  lg: 'max-w-[calc(100%-1rem)] sm:max-w-lg',
  xl: 'max-w-[calc(100%-1rem)] sm:max-w-xl',
  '2xl': 'max-w-[calc(100%-1rem)] sm:max-w-2xl',
  '3xl': 'max-w-[calc(100%-1rem)] sm:max-w-3xl',
} as const;

/**
 * 动画配置 - Overlay（背景遮罩）
 */
export const OVERLAY_ANIMATION = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { duration: 0.2, ease: 'easeInOut' },
} as const;

/**
 * 动画配置 - Content（内容容器）
 */
export const CONTENT_ANIMATION = {
  initial: { opacity: 0, scale: 0.6 },
  animate: { opacity: 1, scale: 1 },
  exit: { opacity: 0, scale: 0.6 },
  transition: { type: 'spring', damping: 18 },
} as const;

/**
 * 默认装饰配置
 */
export const DEFAULT_DECORATIONS: Required<DecorationConfig> = {
  topRight: true,
  bottomLeft: true,
  halftone: true,
} as const;

/**
 * 漫画风格 - 半调纹理样式
 */
export const HALFTONE_STYLE = {
  backgroundImage: 'radial-gradient(circle, rgba(0,0,0,0.2) 8%, transparent 10%)',
  backgroundSize: '18px 18px',
} as const;

/**
 * 漫画风格 - 标题文本阴影
 */
export const TITLE_TEXT_SHADOW = '3px 3px 0 #FACC15';

/**
 * 漫画风格 - 描边文本阴影（白色）
 */
export const STROKE_TEXT_SHADOW_WHITE = '-1px 0 #fff, 1px 0 #fff, 0 1px #fff, 0 -1px #fff';

/**
 * 漫画风格 - 描边文本阴影（黑色）
 */
export const STROKE_TEXT_SHADOW_BLACK = '-1px 0 #000, 1px 0 #000, 0 1px #000, 0 -1px #000';
