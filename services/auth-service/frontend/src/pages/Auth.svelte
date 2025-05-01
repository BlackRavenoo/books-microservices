<script lang="ts">
    import { onMount } from 'svelte';
    import { userInfo, logout } from '../libs/auth';

    export let AppName: string = "Authenticator";
    export let errorCode: number = 404;
    export let Fingerprint: string = "";

    const _unused = { errorCode, Fingerprint };

    let errorMessage: string;
    let serviceName: string = "AuthLib";

    let name = $userInfo?.name || "User";
    let avatarPath = $userInfo?.avatar || "https://static.wikitide.net/hololivewiki/4/4e/Vestia_Zeta_-_Portrait_01.png";
</script>

<svelte:head>
    <title>Аутентификация | {AppName}</title>
</svelte:head>

<div class="modal-container login-container">
    <h2>Авторизация</h2>
    {#if errorMessage}
        <p class="error">{errorMessage}</p>
    {/if}
    <div class="content">
        <span class="content-description">Приложение <b>«{serviceName}»</b> запрашивает разрешение на доступ к учётной записи</span>
        <div class="account">
            <img src="{avatarPath}" alt="{name}" class="avatar">
            <span class="nickname">{name}</span>
        </div>
        <button class="action">Продолжить как {name}</button>
        <a href="/login" class="sub_href cancel">Войти в другой аккаунт</a>
    </div>
</div>

<style>
    .action {
        margin-top: 0;
    }

    .avatar {
        margin-right: 10px;
    }

    .nickname {
        display: inline-block;
        font-size: 1.1rem;
        font-weight: 600;
        color: var(--text-color);
        vertical-align: middle;
        max-width: calc(100% - 70px);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
    
    .error {
        color: red;
        font-size: 0.9rem;
        margin: 5px 0;
    }
</style>