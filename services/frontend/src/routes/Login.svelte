<script lang="ts">
    import { startAuth } from '../utils/auth';
    
    function handleLogin() {
        startAuth();
    }
</script>

<div class="container">
    <div class="login-container">
        <h1>Вход в систему</h1>
        <p>Для доступа к расширенным функциям библиотеки, пожалуйста, войдите в свой аккаунт.</p>
        
        <button class="login-button" on:click={handleLogin}>
            Войти через OAuth сервис
        </button>
        
        <p class="login-info">
            Вы будете перенаправлены на страницу авторизации нашего сервиса.
        </p>
    </div>
</div>