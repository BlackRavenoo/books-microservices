<script lang="ts">
    import { onMount } from 'svelte';
    import { createEventDispatcher } from 'svelte';
    import { checkAuthStatus } from '../libs/auth';
    
    const dispatch = createEventDispatcher();

    let email = '';
    let password = '';
    let errorMessage = '';
    
    function parseUrlParams() {
        const params = new URLSearchParams(window.location.search);
        const loginError = params.get('login_error');
        if (loginError) {
            errorMessage = decodeURIComponent(loginError);
        }
    }
    
    onMount(() => {
        parseUrlParams();
    });
    
    const reset: () => void = () => {
        dispatch('resetPassword');
    }
</script>

<form action="/auth/login" method="post">
    <input type="email" name="email" placeholder="Email" bind:value={email} required>
    <input type="password" name="password" placeholder="Пароль" bind:value={password} required>
    {#if errorMessage}
        <p class="error">{errorMessage}</p>
    {/if}
    <button type="submit" class="action">Войти</button>
</form>
<span class="sub_href">Забыли пароль?<button type="button" on:click={reset}>Восстановить</button></span>

<style scoped>
    form {
        width: 100%;
    }

    input {
        margin-bottom: var(--padding-sm);
    }

    .sub_href {
        margin-top: calc(var(--padding-lg) * -1);
    }

    .sub_href:hover, .sub_href:focus {
        filter: brightness(1);
    }

    .sub_href button {
        background-color: transparent;
        color: var(--primary-color);
        border: none;
        cursor: pointer;
    }

    .sub_href button:hover {
        text-decoration: underline;
    }

    .error {
        color: red;
        font-size: 0.9rem;
        margin: 5px 0;
    }
</style>