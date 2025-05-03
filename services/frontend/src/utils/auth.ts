import type { Token, User } from "../store/authStore";
import FingerprintJS from '@fingerprintjs/fingerprintjs';

export const AUTH_CONFIG = {
    clientId: 'book-app',
    authorizationEndpoint: 'http://127.0.0.1:5001/oauth/authorize',
    tokenEndpoint: '/oauth/token',
    userInfoEndpoint: '/oauth/me',
    redirectUri: window.location.origin + '/callback',
    scope: ''
};

const fpPromise = FingerprintJS.load();

export async function getFingerprint(): Promise<string> {
    try {
        const fp = await fpPromise;
        const result = await fp.get();
        return result.visitorId;
    } catch (error) {
        console.error('Error generating fingerprint:', error);
        throw new Error('Unable to generate device fingerprint');
    }
}

export function generateRandomString(length: number): string {
    let text = '';
    const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~';
    for (let i = 0; i < length; i++) {
        text += possible.charAt(Math.floor(Math.random() * possible.length));
    }
    return text;
}

async function sha256(plain: string): Promise<ArrayBuffer> {
    const encoder = new TextEncoder();
    const data = encoder.encode(plain);
    return await window.crypto.subtle.digest('SHA-256', data);
}

function base64URLEncode(buffer: ArrayBuffer): string {
    return btoa(String.fromCharCode(...new Uint8Array(buffer)))
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=+$/, '');
}

export async function generateCodeChallenge(codeVerifier: string): Promise<string> {
    const hashed = await sha256(codeVerifier);
    return base64URLEncode(hashed);
}

export async function startAuth() {
    const codeVerifier = generateRandomString(128);
    localStorage.setItem('codeVerifier', codeVerifier);
    
    const codeChallenge = await generateCodeChallenge(codeVerifier);

    try {
        const fingerprint = await getFingerprint();
        localStorage.setItem('fingerprint', fingerprint);
        
        const authUrl = new URL(AUTH_CONFIG.authorizationEndpoint);
        authUrl.searchParams.append('client_id', AUTH_CONFIG.clientId);
        authUrl.searchParams.append('redirect_uri', AUTH_CONFIG.redirectUri);
        authUrl.searchParams.append('response_type', 'code');
        authUrl.searchParams.append('scope', AUTH_CONFIG.scope);
        authUrl.searchParams.append('code_challenge_method', 'S256');
        authUrl.searchParams.append('code_challenge', codeChallenge);
        authUrl.searchParams.append('fingerprint', fingerprint);
        
        window.location.href = authUrl.toString();
    } catch (error) {
        console.error('Authentication error:', error);
        alert('Случилась ошибка при генерации отпечка пользователя.');
    }
}

export async function exchangeCodeForTokens(code: string): Promise<Token> {
    const codeVerifier = localStorage.getItem('codeVerifier');
    const fingerprint = localStorage.getItem('fingerprint');
    
    if (!codeVerifier) {
        throw new Error('No code verifier found');
    }

    if (!fingerprint) {
        throw new Error('No device fingerprint found');
    }
    
    const params = new URLSearchParams();
    params.append('client_id', AUTH_CONFIG.clientId);
    params.append('grant_type', 'authorization_code');
    params.append('code', code);
    params.append('redirect_uri', AUTH_CONFIG.redirectUri);
    params.append('code_verifier', codeVerifier);
    params.append('fingerprint', fingerprint);
    
    const response = await fetch(AUTH_CONFIG.tokenEndpoint, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: params
    });
    
    if (!response.ok) {
        throw new Error('Failed to exchange code for tokens');
    }
    
    return await response.json();
}

export async function getUserInfo(token: Token): Promise<User> {
    const response = await fetch(AUTH_CONFIG.userInfoEndpoint, {
        headers: {
            'Authorization': `${token.token_type} ${token.access_token}`
        }
    });
    
    if (!response.ok) {
        throw new Error('Failed to fetch user info');
    }
    
    return await response.json();
}

export function isTokenExpired(token: Token): boolean {
    if (!token?.access_token) return true;
    
    try {
        const payload = JSON.parse(atob(token.access_token.split('.')[1]));
        return Date.now() >= payload.exp * 1000;
    } catch (e) {
        return true;
    }
}

export async function refreshAccessToken(refresh_token: string): Promise<Token> {
    const fingerprint = localStorage.getItem('fingerprint');
    
    if (!fingerprint) {
        throw new Error('No device fingerprint found');
    }

    const params = new URLSearchParams();
    params.append('client_id', AUTH_CONFIG.clientId);
    params.append('grant_type', 'refresh_token');
    params.append('refresh_token', refresh_token);
    params.append('fingerprint', fingerprint);
    
    const response = await fetch(AUTH_CONFIG.tokenEndpoint, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: params
    });
    
    if (!response.ok) {
        throw new Error('Failed to refresh token');
    }
    
    return await response.json();
}