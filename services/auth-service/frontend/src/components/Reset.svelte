<script lang="ts">
    import { resetPassword } from '../libs/api';
    
    let email = '';
    let errorMessage = '';
    let successMessage = '';
    let loading = false;
    
    const reset: (event: Event) => void = async (event) => {
        event.preventDefault();
        
        if (!email) {
            errorMessage = 'Пожалуйста, введите email';
            return;
        }
        
        loading = true;
        errorMessage = '';
        successMessage = '';
        
        try {
            await resetPassword({ email });
            successMessage = 'Инструкции по сбросу пароля отправлены на вашу почту';
            email = '';
        } catch (error) {
            errorMessage = error.message;
            console.error('Ошибка сброса пароля:', error);
        } finally {
            loading = false;
        }
    }
</script>

<form on:submit={reset}>
    <input type="email" name="email" placeholder="Email" bind:value={email} required>
    {#if errorMessage}
        <p class="error">{errorMessage}</p>
    {/if}
    {#if successMessage}
        <p class="success">{successMessage}</p>
    {/if}
    <button type="submit" class="action" disabled={loading}>
        {loading ? 'Отправка...' : 'Отправить код'}
    </button>
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
    
    .success {
        color: green;
        font-size: 0.9rem;
        margin: 5px 0;
    }
</style>