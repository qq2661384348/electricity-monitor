/**
 * ComicModal - 漫画风格模态框组件库
 * 
 * 提供两种使用方式：
 * 1. 便捷模式 - 使用 ComicModal 组件
 * 2. Compound 模式 - 使用 ComicModal.Root, ComicModal.Content 等子组件
 * 
 * @example
 * // 便捷模式
 * import { ComicModal } from '@/components/ui/comic-modal';
 * 
 * <ComicModal isOpen={isOpen} onClose={onClose} title="标题">
 *   内容
 * </ComicModal>
 * 
 * @example
 * // Compound 模式
 * import { ComicModal } from '@/components/ui/comic-modal';
 * 
 * <ComicModal.Root isOpen={isOpen} onClose={onClose}>
 *   <ComicModal.Portal>
 *     <ComicModal.Overlay />
 *     <ComicModal.Content>
 *       <ComicModal.Header>标题</ComicModal.Header>
 *       <ComicModal.Body>内容</ComicModal.Body>
 *     </ComicModal.Content>
 *   </ComicModal.Portal>
 * </ComicModal.Root>
 */

// 主组件导出
export { ComicModal } from './ComicModal';

// 子组件单独导出（用于完全自定义）
export { ComicModalRoot } from './ComicModalRoot';
export { ComicModalPortal } from './ComicModalPortal';
export { ComicModalOverlay } from './ComicModalOverlay';
export { ComicModalContent } from './ComicModalContent';
export { ComicModalHeader } from './ComicModalHeader';
export { ComicModalBody } from './ComicModalBody';
export { ComicModalFooter } from './ComicModalFooter';
export { ComicModalClose } from './ComicModalClose';

// Context 和 Hook 导出
export { ComicModalContext, useComicModalContext } from './ComicModalContext';

// 类型导出
export type {
  ModalSize,
  DecorationConfig,
  ComicModalContextValue,
  ComicModalProps,
  ComicModalRootProps,
  ComicModalOverlayProps,
  ComicModalContentProps,
  ComicModalHeaderProps,
  ComicModalBodyProps,
  ComicModalFooterProps,
  ComicModalCloseProps,
} from './types';

// 常量导出（可选）
export {
  MODAL_SIZE_CLASSES,
  OVERLAY_ANIMATION,
  CONTENT_ANIMATION,
  DEFAULT_DECORATIONS,
  HALFTONE_STYLE,
  TITLE_TEXT_SHADOW,
  STROKE_TEXT_SHADOW_WHITE,
  STROKE_TEXT_SHADOW_BLACK,
} from './constants';
