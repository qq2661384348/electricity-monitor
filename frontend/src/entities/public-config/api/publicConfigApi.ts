import httpClient from '@/shared/api/http-client';

export type PublicCaptchaType = 'string' | 'math' | 'digit';

export interface PublicNotificationConfig {
  qq_bot_public_qq_number: string;
  admin_qq_number: string;
}

export interface PublicCaptchaConfig {
  api_url: string;
  request_timeout_seconds: number;
  token_expire_seconds: number;
  captcha_type: PublicCaptchaType;
  width: number;
  height: number;
  options: number;
}

export interface PublicVerificationConfig {
  code_length: number;
  expire_seconds: number;
}

export interface PublicConfig {
  notification: PublicNotificationConfig;
  captcha: PublicCaptchaConfig;
  verification: PublicVerificationConfig;
}

export const fallbackPublicConfig: PublicConfig = {
  notification: {
    qq_bot_public_qq_number: '',
    admin_qq_number: '',
  },
  captcha: {
    api_url: 'https://v2.xxapi.cn/api/captcha',
    request_timeout_seconds: 5,
    token_expire_seconds: 60,
    captcha_type: 'math',
    width: 300,
    height: 100,
    options: 2,
  },
  verification: {
    code_length: 6,
    expire_seconds: 300,
  },
};

export const publicConfigApi = {
  async getPublicConfig(): Promise<PublicConfig> {
    const { data } = await httpClient.get<PublicConfig>('/public-config');
    return data;
  },
};
