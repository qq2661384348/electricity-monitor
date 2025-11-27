import * as React from "react"
import { cn } from "@/lib/utils"

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, type, error, ...props }, ref) => {
    return (
      <input
        type={type}
        className={cn(
          // 基础样式 - 触控友好 (min-h-[44px] 满足 Apple HIG 触控最小要求)
          "flex min-h-[44px] sm:min-h-[40px] w-full bg-white px-3 py-2 text-base sm:text-sm text-black font-bold",
          // 边框和阴影
          "border-4 border-black outline-none transition-all",
          "shadow-[4px_4px_0_0_rgba(0,0,0,0.5)]",
          // 焦点状态
          "focus-visible:shadow-[8px_8px_0_0_var(--color-brand-primary)] focus-visible:scale-[1.02]",
          // 占位符和禁用状态
          "placeholder:text-gray-500 disabled:cursor-not-allowed disabled:opacity-50",
          // 漫画字体
          "font-comic",
          // 错误状态
          error && "border-status-danger focus-visible:shadow-[8px_8px_0_0_var(--color-status-danger)]",
          className
        )}
        ref={ref}
        {...props}
      />
    )
  }
)
Input.displayName = "Input"

export { Input }
