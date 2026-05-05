import httpClient from '@/shared/api/http-client';
import type { LoginResponse, User } from '@/types';

export const authApi = {
  async sendVerificationCode(qqNumber: string, captchaToken: string): Promise<void> {
    await httpClient.post('/auth/send-verification-code', {
      qq_number: qqNumber,
      captcha_token: captchaToken,
    });
  },

  async verifyAndLogin(qqNumber: string, code: string): Promise<LoginResponse> {
    const { data } = await httpClient.post<LoginResponse>('/auth/verify-and-login', {
      qq_number: qqNumber,
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
