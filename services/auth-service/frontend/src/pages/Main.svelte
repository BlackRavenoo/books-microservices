<script lang="ts">
    import { onMount } from "svelte";
    import { writable, type Writable } from "svelte/store";

    import Login from "../components/Login.svelte";
    import Registration from "../components/Registration.svelte";
    import Reset from "../components/Reset.svelte";

    export let AppName: string = "Authenticator";
    export let errorCode: number;
    export let Fingerprint: string = "";

    const _unused = { errorCode, Fingerprint };

    enum pages {
        login = 0,
        registration = 1,
        reset = 2
    }

    let activePage: Writable<pages> = writable(pages.login);
    let errorMessage: string;

    onMount(() => {
        parseUrlParams();
        window.addEventListener('popstate', parseUrlParams);
        return () => {
            window.removeEventListener('popstate', parseUrlParams);
        };
    });

    function parseUrlParams() {
        const urlParams = new URLSearchParams(window.location.search);
        const page = urlParams.get('page') || 'login';
        
        switch (page) {
            case 'registration':
                changePage(pages.registration);
                break;
            case 'reset':
                changePage(pages.reset);
                break;
            default:
                changePage(pages.login);
        }
    }

    const changePage: (pages) => void = (page: pages) => {
        activePage.set(page);

        const pageParam = page === pages.login ? 'login' : 
                         page === pages.registration ? 'registration' : 'reset';

        const url = new URL(window.location.href);
        url.searchParams.set('page', pageParam);
        history.pushState({}, '', url);
        
        const buttons = document.querySelectorAll(".action_type");
        buttons.forEach((button) => {
            button.classList.remove("active");
        });
        buttons[page].classList.add("active");
    };
</script>

<svelte:head>
    <title>{$activePage === pages.login ? 
            "Вход" : 
            $activePage === pages.registration ?
            "Регистрация" : "Сброс пароля"} | {AppName}</title>
</svelte:head>

<div class="modal-container login-container">
    {#if errorMessage}
        <p class="error">{errorMessage}</p>
    {/if}
    <div class="content">
        <nav>
            <button class="action_type type_0 active" on:click={ () => changePage(pages.login) }>Вход</button>
            <button class="action_type type_1" on:click={ () => changePage(pages.registration) }>Регистрация</button>
            <button class="action_type type_2" on:click={ () => changePage(pages.reset) }>Сброс</button>
        </nav>
        <div id="form">
            {#if $activePage === pages.login}
                <Login on:resetPassword={ () => changePage(pages.reset) } />
            {:else if $activePage === pages.registration}
                <Registration />
            {:else if $activePage === pages.reset}
                <Reset />
            {/if}
        </div>
    </div>
</div>

<style scoped>
    #form {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: var(--padding-sm);
        flex-direction: column;
        width: 100%;
    }

    nav {
        display: flex;
        justify-content: center;
        flex-wrap: wrap;
        margin-bottom: 20px;
    }

    .action_type {
        background-color: transparent;
        color: var(--gray-text-color);
        border-bottom: 2px solid var(--gray-text-color);
        border-radius: 0;
        padding: var(--padding-xs) var(--padding-sm);
        transition: color 0.2s ease, border-bottom 0.2s ease;
    }

    .action_type:hover, .action_type:focus {
        color: var(--text-color);
        border-bottom: 2px solid var(--text-color);
    }

    .action_type:active, .active {
        color: var(--primary-color) !important;
        border-bottom: 2px solid var(--primary-color) !important;
    }
</style>