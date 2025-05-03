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
            
            set({ user, token });
        },
        isAuthenticated: () => {
            const state = get(authStore);
            return state.user !== null && state.token !== null;
        },
        isAdmin: () => {
            const state = get(authStore);
            return state.user?.roles?.includes('admin') || false;
        }
    };
}

export const authStore = createAuthStore();