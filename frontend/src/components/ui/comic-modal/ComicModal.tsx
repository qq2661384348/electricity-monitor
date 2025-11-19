import { ComicModalRoot } from './ComicModalRoot';
import { ComicModalPortal } from './ComicModalPortal';
import { ComicModalOverlay } from './ComicModalOverlay';
import { ComicModalContent } from './ComicModalContent';
import { ComicModalHeader } from './ComicModalHeader';
import { ComicModalBody } from './ComicModalBody';
import { ComicModalFooter } from './ComicModalFooter';
import { ComicModalClose } from './ComicModalClose';
import type { ComicModalProps } from './types';

/**
 * ComicModal - 便捷包装组件
 * 
 * 组合所有子组件，提供简化的 API，适合大部分常见场景
 * 如需完全自定义，请使用 ComicModal.Root 等子组件
 * 
 * @example
 * ```tsx
 * // 基础用法
 * <ComicModal isOpen={isOpen} onClose={onClose} title="标题">
 *   <p>内容</p>
 * </ComicModal>
 * 
 * // 带底部按钮
 * <ComicModal 
 *   isOpen={isOpen} 
 *   onClose={onClose} 
 *   title="确认"
 *   footer={
 *     <>
 *       <Button onClick={onCancel}>取消</Button>
 *       <Button onClick={onConfirm}>确认</Button>
 *     </>
 *   }
 * >
 *   <p>确定要删除吗？</p>
 * </ComicModal>
 * ```
 */
export function ComicModal({
  isOpen,
  onClose,
  size = 'md',
  decorations,
  title,
  showCloseButton = true,
  children,
  footer,
}: ComicModalProps) {
  return (
    <ComicModalRoot isOpen={isOpen} onClose={onClose} size={size} decorations={decorations}>
      <ComicModalPortal>
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <ComicModalOverlay />
          <ComicModalContent>
            {title && (
              <ComicModalHeader showCloseButton={showCloseButton}>
                {title}
              </ComicModalHeader>
            )}
            {!title && showCloseButton && <ComicModalClose />}
            <ComicModalBody>{children}</ComicModalBody>
            {footer && <ComicModalFooter>{footer}</ComicModalFooter>}
          </ComicModalContent>
        </div>
      </ComicModalPortal>
    </ComicModalRoot>
  );
}

// 命名空间导出 - Compound Components 模式
ComicModal.Root = ComicModalRoot;
ComicModal.Portal = ComicModalPortal;
ComicModal.Overlay = ComicModalOverlay;
ComicModal.Content = ComicModalContent;
ComicModal.Header = ComicModalHeader;
ComicModal.Body = ComicModalBody;
ComicModal.Footer = ComicModalFooter;
ComicModal.Close = ComicModalClose;

ComicModal.displayName = 'ComicModal';
