/**
 * 第三方验证码服务
 * 
 * 采用"客户端直连获取、网关代理校验"架构
 * 默认第三方API来自公开运行时配置，不能在前端流程中另行硬编码。
 */

import axios from 'axios';
import { fallbackPublicConfig, type PublicCaptchaConfig, type PublicCaptchaType } from '@/entities/public-config';
import httpClient from '@/shared/api/http-client';

/**
 * 验证码类型
 */
export type CaptchaType = 'string' | 'math' | 'digit';

/**
 * 验证码难度等级
 */
export type CaptchaOptions = 1 | 2 | 3;

/**
 * 生成验证码请求参数
 */
export interface GenerateCaptchaParams {
  type: CaptchaType;
  width?: number;  // 默认 280px
  height?: number; // 默认 80px
  length?: number; // 字符验证码长度，默认 6
  options?: CaptchaOptions; // 难度等级，默认 2
}

/**
 * 生成验证码响应
 */
export interface GenerateCaptchaResponse {
  code: number;
  msg: string;
  data: {
    id: string;   // 验证码唯一ID
    url: string;  // Base64格式的验证码图片
  };
}

/**
 * 校验验证码请求参数
 */
export interface VerifyCaptchaParams {
  id: string;    // 验证码ID
  key: string;   // 用户输入的答案
  type: CaptchaType;
}

/**
 * 校验验证码响应（后端网关标准化响应）
 */
export interface VerifyCaptchaResponse {
  success: boolean;
  message: string;
  code: string;
  token?: string;  // 验证成功时返回的一次性token
}

/**
 * 验证码服务类
 */
class CaptchaService {
  private normalizeCaptchaType(type: string): PublicCaptchaType {
    if (type === 'string' || type === 'math' || type === 'digit') {
      return type;
    }

    return fallbackPublicConfig.captcha.captcha_type;
  }

  /**
   * 按公开运行时配置生成验证码。
   * 前端直连第三方生成接口，后端使用同一份配置代理校验。
   */
  async generateConfiguredCaptcha(
    config: PublicCaptchaConfig = fallbackPublicConfig.captcha,
  ): Promise<GenerateCaptchaResponse> {
    try {
      const response = await axios.get<GenerateCaptchaResponse>(
        config.api_url,
        {
          params: {
            type: this.normalizeCaptchaType(config.captcha_type),
            width: config.width,
            height: config.height,
            options: config.options,
          },
          timeout: config.request_timeout_seconds * 1000,
        }
      );

      if (response.data.code !== 200) {
        throw new Error(response.data.msg || '获取验证码失败');
      }

      return response.data;
    } catch (error) {
      // 网络错误或超时
      if (axios.isAxiosError(error)) {
        if (error.code === 'ECONNABORTED') {
          throw new Error('验证码服务请求超时', { cause: error });
        }
        if (!error.response) {
          throw new Error('验证码服务暂时不可用', { cause: error });
        }
      }
      throw error;
    }
  }

  /**
   * 保留旧入口，默认使用运行时模板中的算数验证码参数。
   */
  async generateMathCaptcha(): Promise<GenerateCaptchaResponse> {
    return this.generateConfiguredCaptcha();
  }

  /**
   * 生成自定义验证码
   * @param params 验证码参数
   */
  async generateCaptcha(params: GenerateCaptchaParams): Promise<GenerateCaptchaResponse> {
    try {
      const response = await axios.get<GenerateCaptchaResponse>(
        fallbackPublicConfig.captcha.api_url,
        {
          params,
          timeout: fallbackPublicConfig.captcha.request_timeout_seconds * 1000,
        }
      );

      if (response.data.code !== 200) {
        throw new Error(response.data.msg || '获取验证码失败');
      }

      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        if (error.code === 'ECONNABORTED') {
          throw new Error('验证码服务请求超时', { cause: error });
        }
        if (!error.response) {
          throw new Error('验证码服务暂时不可用', { cause: error });
        }
      }
      throw error;
    }
  }

  /**
   * 校验验证码
   * 通过后端网关代理校验
   */
  async verifyCaptcha(params: VerifyCaptchaParams): Promise<VerifyCaptchaResponse> {
    try {
      const response = await httpClient.post<VerifyCaptchaResponse>(
        '/captcha/verify',
        params
      );

      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error)) {
        if (error.response?.status === 400) {
          // 验证失败
          return {
            success: false,
            message: error.response.data?.message || '验证码错误',
            code: 'VERIFY_FAILED',
          };
        }
        if (error.response?.status === 503) {
          // 服务不可用
          return {
            success: false,
            message: '验证服务暂时不可用',
            code: 'SERVICE_UNAVAILABLE',
          };
        }
      }
      // 其他错误
      return {
        success: false,
        message: '验证码校验失败',
        code: 'UNKNOWN_ERROR',
      };
    }
  }

  /**
   * 刷新验证码（生成新的验证码）
   * 这是一个便捷方法，直接返回算数验证码
   */
  async refreshCaptcha(): Promise<{
    id: string;
    imageUrl: string;
  }> {
    const response = await this.generateConfiguredCaptcha();
    return {
      id: response.data.id,
      imageUrl: response.data.url,
    };
  }
}

// 导出单例
export const captchaService = new CaptchaService();
