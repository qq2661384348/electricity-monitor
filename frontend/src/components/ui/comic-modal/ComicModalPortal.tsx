import { createPortal } from 'react-dom';
import type { ReactNode } from 'react';

interface ComicModalPortalProps {
  readonly children: ReactNode;
}

/**
 * ComicModalPortal - 使用 React Portal 将模态框渲染到 document.body
 * 
 * 这样可以避免 z-index 和定位问题
 * 
 * @example
 * ```tsx
 * <ComicModalPortal>
 *   <ComicModalOverlay />
 *   <ComicModalContent>...</ComicModalContent>
 * </ComicModalPortal>
 * ```
 */
export function ComicModalPortal({ children }: Readonly<ComicModalPortalProps>) {
  if (typeof document === 'undefined') {
    return null;
  }
  
  return createPortal(children, document.body);
}

ComicModalPortal.displayName = 'ComicModalPortal';
