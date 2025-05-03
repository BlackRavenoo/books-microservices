<script lang="ts">
    import { onMount } from 'svelte';
    import { navigate } from 'svelte-routing';
    import { exchangeCodeForTokens, getUserInfo } from '../utils/auth';
    import { authStore } from '../store/authStore';
    
    let error = '';
    
    onMount(async () => {
        const urlParams = new URLSearchParams(window.location.search);
        const code = urlParams.get('code');
        
        if (code) {
            try {
                const token = await exchangeCodeForTokens(code);
                
                authStore.setTokens(token.access_token, token.refresh_token, token.token_type);
                
                const user = await getUserInfo(token);
                
                authStore.setUser(user);
                
                navigate('/');
            } catch (e) {
                console.error('Auth error:', e);
                error = 'Authentication failed. Please try again.';
            }
        } else if (urlParams.get('error')) {
            error = urlParams.get('error_description') || 'Authentication failed';
        } else {
            error = 'Invalid callback URL';
        }
    });
</script>

<div class="container">
    <div class="auth-callback">
        {#if error}
            <div class="error-message">
                <h2>Ошибка аутентификации</h2>
                <p>{error}</p>
                <button on:click={() => navigate('/')}>Вернуться на главную</button>
            </div>
        {:else}
            <div class="loading">
                <h2>Авторизация...</h2>
                <div class="spinner"></div>
                <p>Пожалуйста, подождите пока мы завершим процесс аутентификации.</p>
            </div>
        {/if}
    </div>
</div>

<style>
    .container {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 2rem 1rem;
    }
    
    .auth-callback {
        max-width: 500px;
        margin: 2rem auto;
        padding: 2rem;
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        text-align: center;
    }
    
    .error-message {
        color: #ef4444;
    }
    
    .loading {
        display: flex;
        flex-direction: column;
        align-items: center;
    }
    
    .spinner {
        border: 4px solid rgba(255, 255, 255, 0.1);
        border-left-color: var(--primary-color);
        border-radius: 50%;
        width: 40px;
        height: 40px;
        animation: spin 1s linear infinite;
        margin: 1rem 0;
    }
    
    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
    
    button {
        margin-top: 1rem;
        padding: 0.5rem 1rem;
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 0.25rem;
        cursor: pointer;
    }
</style>