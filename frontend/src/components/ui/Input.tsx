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
          "flex h-10 w-full bg-white px-3 py-2 text-sm text-black font-bold border-4 border-black outline-none transition-all placeholder:text-gray-500 focus-visible:shadow-[8px_8px_0_0_var(--color-brand-primary)] focus-visible:scale-[1.02] shadow-[4px_4px_0_0_rgba(0,0,0,0.5)] disabled:cursor-not-allowed disabled:opacity-50",
          "font-comic", // 确保使用漫画字体
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
