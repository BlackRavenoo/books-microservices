<script lang="ts">
    export let AppName: string = "Authenticator";
    export let Fingerprint: string = "";
    export let errorCode: number = 404;

    const _unused = Fingerprint;

    const HttpErrorMessages: Record<number, string> = {
        400: "Некорректный запрос",
        401: "Требуется авторизация",
        403: "Доступ запрещен",
        404: "Страница не найдена",
        500: "Внутренняя ошибка сервера",
        502: "Ошибка шлюза",
        503: "Сервис недоступен",
        504: "Превышение времени ожидания"
    };

    let errorMessage: string;

    $: {
        errorMessage = HttpErrorMessages[errorCode] || "Произошла непредвиденная ошибка";
    }
</script>

<svelte:head>
    <title>Ошибка {errorCode} | {AppName}</title>
</svelte:head>

<div class="modal-container error-container">
    <div class="content">
        <div class="error-icon">
            <span>{errorCode}</span>
        </div>
        
        <p class="error-message">{errorMessage}</p>
        
        <span class="content-description">
            Произошла ошибка при обработке вашего запроса
        </span>
        
        <button class="action" on:click={() => window.location.hash = '/main'}>
            Вернуться на главную
        </button>
    </div>
</div>

<style>
    .error-container {
        text-align: center;
    }
    
    .error-icon {
        display: flex;
        justify-content: center;
        align-items: center;
        width: 120px;
        height: 120px;
        margin: var(--padding-sm) auto;
        background-color: var(--font-gray);
        border-radius: 50%;
        font-size: 2.5rem;
        font-weight: 700;
        color: var(--primary-color);
    }
    
    .error-message {
        font-size: 1.3rem;
        font-weight: 600;
        color: var(--text-color);
        margin-bottom: var(--padding-sm);
    }
    
    .content-description {
        font-size: 0.85rem;
        font-weight: 400;
        color: var(--gray-text-color);
        margin-bottom: var(--padding-sm);
    }
    
    .action {
        width: 100%;
        background-color: var(--primary-color);
        color: var(--white);
    }
</style>