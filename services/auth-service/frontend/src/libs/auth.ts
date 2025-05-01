import { writable } from 'svelte/store';

export interface UserInfo {
    id: number,
    name: string,
    email: string,
    avatar?: string; 
}

export const isAuthenticated = writable<boolean>(false);
export const userInfo = writable<UserInfo | null>(null);

export async function checkAuthStatus(): Promise<void> {
    try {
        const response = await fetch('/api/user/me', {
            credentials: 'include'
        });
        
        if (response.ok) {
            const userData = await response.json();
            isAuthenticated.set(true);
            userInfo.set(userData);
        } else {
            isAuthenticated.set(false);
            userInfo.set(null);
        }
    } catch (error) {
        console.error('Auth check failed:', error);
        isAuthenticated.set(false);
        userInfo.set(null);
    }
}

export function logout(): void {
    window.location.href = '/auth/logout';
}