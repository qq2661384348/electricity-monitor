import httpClient from '@/shared/api/http-client';
import type { LoginMode, LoginResponse, User } from '@/types';

export const authApi = {
  async sendVerificationCode(
    loginMode: LoginMode,
    identifier: string,
    captchaToken: string,
  ): Promise<void> {
    await httpClient.post('/auth/send-verification-code', {
      login_mode: loginMode,
      identifier,
      ...(loginMode === 'qq' ? { qq_number: identifier } : { email: identifier }),
      captcha_token: captchaToken,
    });
  },

  async verifyAndLogin(
    loginMode: LoginMode,
    identifier: string,
    code: string,
  ): Promise<LoginResponse> {
    const { data } = await httpClient.post<LoginResponse>('/auth/verify-and-login', {
      login_mode: loginMode,
      identifier,
      ...(loginMode === 'qq' ? { qq_number: identifier } : { email: identifier }),
      code,
    });
    return data;
  },

  async refreshSession(): Promise<LoginResponse> {
    const { data } = await httpClient.post<LoginResponse>('/auth/refresh');
    return data;
  },

  async getCurrentUser(): Promise<User> {
    const { data } = await httpClient.get<User>('/auth/me');
    return data;
  },

  async logout(): Promise<void> {
    await httpClient.post('/auth/logout');
  },
};
