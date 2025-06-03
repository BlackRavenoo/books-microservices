import { get, writable } from 'svelte/store';

export interface User {
    id: string;
    username: string;
    roles: string[];
    avatar_url: string;
}

export interface Token {
    access_token: string;
    refresh_token: string;
    token_type: string;
}

interface AuthState {
    user: User | null;
    token: Token | null;
}

function createAuthStore() {
    const { subscribe, set, update } = writable<AuthState>({
        user: null,
        token: null,
    });

    return {
        subscribe,
        setTokens: (access_token: string, refresh_token: string, token_type: string = 'Bearer') => {
            const token = { access_token, refresh_token, token_type };
            localStorage.setItem('token', JSON.stringify(token));
            update(state => ({ ...state, token }));
        },
        setUser: (user: User) => {
            localStorage.setItem('user', JSON.stringify(user));
            update(state => ({ ...state, user }));
        },
        logout: () => {
            localStorage.removeItem('token');
            localStorage.removeItem('user');
            localStorage.removeItem('codeVerifier');
            set({ 
                user: null, 
                token: null,
            });
        },
        initialize: () => {
            const tokenStr = localStorage.getItem('token');
            const userStr = localStorage.getItem('user');
            const token = tokenStr ? JSON.parse(tokenStr) : null;
            const user = userStr ? JSON.parse(userStr) : null;
            
            if (token) {
                try {
                    const payload = JSON.parse(atob(token.access_token.split('.')[1]));
                    if (Date.now() >= payload.exp * 1000) {
                        localStorage.removeItem('token');
                        localStorage.removeItem('user');
                        set({ user: null, token: null });
                        return;
                    }
                } catch (e) {
                    localStorage.removeItem('token');
                    localStorage.removeItem('user');
                    set({ user: null, token: null });
                    return;
                }
            }
            
            set({ user, token });
        },
        isAuthenticated: () => {
            const state = get(authStore);
            return state.user !== null && state.token !== null;
        },
        isAdmin: () => {
            const state = get(authStore);
            return state.user?.roles?.includes('admin') || false;
        },
        isTokenExpired: () => {
            const state = get(authStore);
            if (!state.token?.access_token) return true;
            
            try {
                const payload = JSON.parse(atob(state.token.access_token.split('.')[1]));
                return Date.now() >= payload.exp * 1000;
            } catch (e) {
                return true;
            }
        },
        isTokenExpiringSoon: (minutesBeforeExpiry: number = 2) => {
            const state = get(authStore);
            if (!state.token?.access_token) return true;
            
            try {
                const payload = JSON.parse(atob(state.token.access_token.split('.')[1]));
                const expiryTime = payload.exp * 1000;
                const warningTime = expiryTime - (minutesBeforeExpiry * 60 * 1000);
                return Date.now() >= warningTime;
            } catch (e) {
                return true;
            }
        }
    };
}

export const authStore = createAuthStore();