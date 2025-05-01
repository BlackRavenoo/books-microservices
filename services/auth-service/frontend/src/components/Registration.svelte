<script lang="ts">
    import { onMount } from 'svelte';
    import { registerUser } from '../libs/api';
    import { checkAuthStatus } from '../libs/auth';
    
    let errorMessage = '';
    
    function parseUrlParams() {
        const params = new URLSearchParams(window.location.search);
        const registerError = params.get('register_error');
        if (registerError) {
            errorMessage = decodeURIComponent(registerError);
        }
    }
    
    onMount(() => {
        parseUrlParams();
    });
</script>

<form action="/auth/register" method="post">
    <input type="text" name="name" placeholder="Имя" required>
    <input type="email" name="email" placeholder="Email" required>
    <input type="password" name="password" placeholder="Пароль" required>
    <input type="password" name="password_confirm" placeholder="Повторите пароль" required>
    {#if errorMessage}
        <p class="error">{errorMessage}</p>
    {/if}
    <button type="submit" class="action">Зарегистрироваться</button>
</form>

<style scoped>
    form {
        width: 100%;
    }

    input {
        margin-bottom: var(--padding-sm);
    }

    .error {
        color: red;
        font-size: 0.9rem;
        margin: 5px 0;
    }
</style>