const AUTH_BASE_URL = '/auth';

export interface LoginFormData {
    email: string;
    password: string;
}

export interface RegisterFormData {
    name: string;
    email: string;
    password: string;
    password_confirm: string;
}

export interface ResetPasswordFormData {
    email: string;
}

export async function loginUser(data: LoginFormData): Promise<void> {
    const formData = new FormData();
    formData.append('email', data.email);
    formData.append('password', data.password);
    
    const response = await fetch(`${AUTH_BASE_URL}/login`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: formData,
        redirect: 'follow',
        credentials: 'include'
    });
    
    if (response.redirected) {
        window.location.href = response.url;
    } else if (!response.ok) {
        const error = await response.text();
        throw new Error(error || 'Ошибка при входе');
    }
}

export async function registerUser(data: RegisterFormData): Promise<void> {
    const formData = new FormData();
    formData.append('name', data.name);
    formData.append('email', data.email);
    formData.append('password', data.password);
    formData.append('password_confirm', data.password_confirm);
    
    const response = await fetch(`${AUTH_BASE_URL}/register`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: formData,
        redirect: 'follow',
        credentials: 'include'
    });
    
    if (response.redirected) {
        window.location.href = response.url;
    } else if (!response.ok) {
        const error = await response.text();
        throw new Error(error || 'Ошибка при регистрации');
    }
}

export async function resetPassword(data: ResetPasswordFormData): Promise<void> {
    const formData = new FormData();
    formData.append('email', data.email);
    
    const response = await fetch(`${AUTH_BASE_URL}/reset-password`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: formData,
        redirect: 'follow',
        credentials: 'include'
    });
    
    if (response.redirected) {
        window.location.href = response.url;
    } else if (!response.ok) {
        const error = await response.text();
        throw new Error(error || 'Ошибка при сбросе пароля');
    }
}