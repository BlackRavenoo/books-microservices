import { authStore } from './store/authStore';
import { refreshAccessToken } from './utils/auth';
import { get } from 'svelte/store';

class TokenManager {
    private refreshTimer: number | null = null;
    private unsubscribe: (() => void) | null = null;
    private isRefreshing = false;

    start() {
        this.unsubscribe = authStore.subscribe((state) => {
            if (state.token?.access_token && !this.isRefreshing) {
                this.scheduleTokenRefresh();
            } else if (!state.token?.access_token) {
                this.stopTokenRefresh();
            }
        });
    }

    private scheduleTokenRefresh() {
        this.stopTokenRefresh();
        
        const timeUntilRefresh = this.getTimeUntilRefresh();
        
        if (timeUntilRefresh > 0) {
            console.log(`Токен будет обновлен через ${Math.round(timeUntilRefresh / 1000 / 60)} минут`);
            
            this.refreshTimer = window.setTimeout(async () => {
                await this.refreshToken();
            }, timeUntilRefresh);
        } else if (timeUntilRefresh === 0) {
            this.refreshToken();
        }
    }

    private getTimeUntilRefresh(): number {
        if (authStore.isTokenExpired()) {
            return -1;
        }

        if (authStore.isTokenExpiringSoon(2)) {
            return 0;
        }

        const authState = get(authStore);
        if (!authState.token?.access_token) {
            return -1;
        }

        try {
            const payload = JSON.parse(atob(authState.token.access_token.split('.')[1]));
            const expiryTime = payload.exp * 1000;
            const refreshTime = expiryTime - (2 * 60 * 1000);
            
            return Math.max(0, refreshTime - Date.now());
        } catch (error) {
            console.error('Ошибка при парсинге токена:', error);
            return -1;
        }
    }

    private async refreshToken() {
        if (this.isRefreshing) {
            return;
        }
      
        this.isRefreshing = true;

        try {
            if (!authStore.isTokenExpiringSoon(2)) {
                console.log('Токен еще не нуждается в обновлении');
                this.scheduleTokenRefresh();
                return;
            }

            const authState = get(authStore);
            const refreshToken = authState.token?.refresh_token;

            if (!refreshToken) {
                console.log('Нет refresh токена, выходим из системы');
                authStore.logout();
                return;
            }

            console.log('Обновлям токен...');
            const newToken = await refreshAccessToken(refreshToken);
            
            authStore.setTokens(newToken.access_token, newToken.refresh_token, newToken.token_type);
            console.log('Токен успешно обновлен');
            
            this.scheduleTokenRefresh();
        } catch (error) {
            console.error('Ошибка при обновлении токена:', error);
            authStore.logout();
        } finally {
            this.isRefreshing = false;
        }
    }

    private stopTokenRefresh() {
        if (this.refreshTimer) {
            clearTimeout(this.refreshTimer);
            this.refreshTimer = null;
        }
    }

    destroy() {
        this.stopTokenRefresh();
        if (this.unsubscribe) {
            this.unsubscribe();
        }
    }
}

export const tokenManager = new TokenManager();