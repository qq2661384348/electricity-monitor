import type { ReactNode } from 'react';

/**
 * 模态框尺寸类型
 */
export type ModalSize = 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl';

/**
 * 装饰配置
 */
export interface DecorationConfig {
  /** 显示右上角装饰色块 */
  topRight?: boolean;
  /** 显示左下角装饰色块 */
  bottomLeft?: boolean;
  /** 显示半调纹理背景 */
  halftone?: boolean;
}

/**
 * ComicModal Context 值
 */
export interface ComicModalContextValue {
  /** 模态框是否打开 */
  isOpen: boolean;
  /** 关闭模态框回调 */
  onClose: () => void;
  /** 模态框尺寸 */
  size: ModalSize;
  /** 装饰配置 */
  decorations: DecorationConfig;
}

/**
 * ComicModalRoot Props
 */
export interface ComicModalRootProps {
  /** 是否打开 */
  isOpen: boolean;
  /** 关闭回调 */
  onClose: () => void;
  /** 尺寸 */
  size?: ModalSize;
  /** 装饰配置 */
  decorations?: DecorationConfig;
  /** 子元素 */
  children: ReactNode;
}

/**
 * ComicModalOverlay Props
 */
export interface ComicModalOverlayProps {
  /** 点击遮罩是否关闭 */
  closeOnClick?: boolean;
  /** 自定义类名 */
  className?: string;
}

/**
 * ComicModalContent Props
 */
export interface ComicModalContentProps {
  /** 子元素 */
  children: ReactNode;
  /** 自定义类名 */
  className?: string;
  /** 点击内容区域时阻止冒泡（防止意外关闭） */
  stopPropagation?: boolean;
}

/**
 * ComicModalHeader Props
 */
export interface ComicModalHeaderProps {
  /** 子元素 */
  children: ReactNode;
  /** 是否显示关闭按钮 */
  showCloseButton?: boolean;
  /** 自定义类名 */
  className?: string;
}

/**
 * ComicModalBody Props
 */
export interface ComicModalBodyProps {
  /** 子元素 */
  children: ReactNode;
  /** 自定义类名 */
  className?: string;
}

/**
 * ComicModalFooter Props
 */
export interface ComicModalFooterProps {
  /** 子元素 */
  children: ReactNode;
  /** 自定义类名 */
  className?: string;
}

/**
 * ComicModalClose Props
 */
export interface ComicModalCloseProps {
  /** 自定义类名 */
  className?: string;
  /** aria-label */
  'aria-label'?: string;
}

/**
 * 便捷 Wrapper Props
 */
export interface ComicModalProps extends Omit<ComicModalRootProps, 'children'> {
  /** 标题 */
  title?: string;
  /** 显示关闭按钮 */
  showCloseButton?: boolean;
  /** 子元素（主体内容） */
  children: ReactNode;
  /** 底部按钮区域 */
  footer?: ReactNode;
}
