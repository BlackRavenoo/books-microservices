<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    // @ts-ignore - Ignores the next line only
    import * as FingerprintJS from '@fingerprintjs/fingerprintjs';
    import { writable, type Writable } from 'svelte/store';

    import Main from "./pages/Main.svelte";
    import Auth from "./pages/Auth.svelte";
    import NotFound from "./pages/Error.svelte";

    export let logoPath: string = "/logo.png";
    export let primaryColor: string = "#007bff";
    export let AppName: string = "Authenticator";

    export const theme: Writable<string> = writable("light");
    export const currentPage: Writable<string> = writable("home");
    export const errorCode: Writable<number> = writable(404);

    export const Fingerprint: Writable<string> = writable("");
    let pageComponent: any = Main;
    let logo: HTMLElement;
    let modalContainer: HTMLElement;
    let resizeObserver: ResizeObserver;

    const routes: Record<string, any> = {
        "main": Main,
        "auth": Auth,
    };

    const updatePage: () => void = () => {
        const hash = window.location.hash.slice(2) || "main";
        console.log(hash);
        
        // TODO: fix page url scrapping
        const errorMatch = hash.match(/^error\/(\d+)$/);
        if (errorMatch) {
            const code = parseInt(errorMatch[1], 10);
            errorCode.set(isNaN(code) ? 404 : code);
            currentPage.set("error");
        } else {
            currentPage.set(hash);
        }

        setLogoPosition();
    };

    const setLogoPosition: () => void = () => {
        const logo = document.querySelector(".logo") as HTMLElement;
        if (logo) {
            const modal = document.querySelector(".modal-container") as HTMLElement;
            if (modal)
                logo.style.transform = `translate(-50%, -${modal.clientHeight / 2 + logo.clientHeight / 2 + 40}px)`;
        }
    };

    const toggleTheme: () => void = () => {
        const newTheme = $theme === "light" ? "dark" : "light";
        applyTheme(newTheme);
    };

    const applyTheme: (string) => void = (savedTheme) => {
        theme.set(savedTheme);
        localStorage.setItem("theme", savedTheme);

        let themeStyle = document.getElementById('theme-override');
        if (!themeStyle) {
            themeStyle = document.createElement('style');
            themeStyle.id = 'theme-override';
            document.head.appendChild(themeStyle);
        }

        if (savedTheme === "dark") {
            document.documentElement.classList.add("dark");
            themeStyle.textContent = `
                html.dark * {
                    --white: #2e2e2e !important;
                    --gray: #383838 !important;
                    --font-gray: #403f3f !important;
                    --text-color: #f5f5f5 !important;
                    --gray-text-color: #a0a0a0 !important;
                    --box-shadow: 0px 0px 45px -4px rgba(100, 100, 100, 0.25) !important;
                }
            `;
        } else {
            document.documentElement.classList.remove("dark");
            themeStyle.textContent = `
                * {
                    --white: #FFFFFF !important;
                    --gray: #F2F2F2 !important;
                    --font-gray: #F8F8F8 !important;
                    --text-color: #222222 !important;
                    --gray-text-color: #888888 !important;
                    --box-shadow: 0px 0px 20px -4px rgba(34, 60, 80, 0.2) !important;
                }
            `;
        }
    };

    onMount(() => {
        const savedTheme = localStorage.getItem("theme");

        document.documentElement.style.setProperty('--primary-color', primaryColor);
        
        let prefersDark: boolean;
        prefersDark = savedTheme === null ? 
            window.matchMedia("(prefers-color-scheme: dark)").matches :
            savedTheme === "dark";

        applyTheme(prefersDark ? "dark" : "light");
        
        resizeObserver = new ResizeObserver(() => {
            setLogoPosition();
        });
        
        logo = document.querySelector(".logo");
        modalContainer = document.querySelector(".modal-container") as HTMLElement;
        
        if (logo) resizeObserver.observe(logo);
        if (modalContainer) resizeObserver.observe(modalContainer);
        
        window.addEventListener("resize", setLogoPosition);
        
        setTimeout(setLogoPosition, 100);
        
        updatePage();

        FingerprintJS.load()
            .then(fp => fp.get())
            .then(result => {
                Fingerprint.set(result.visitorId);
            });

        window.addEventListener("hashchange", updatePage);
        
        return () => {
            window.removeEventListener("hashchange", updatePage);
            window.removeEventListener("resize", setLogoPosition);
            if (resizeObserver) {
                resizeObserver.disconnect();
            }
        };
    });

    $: {
        if (pageComponent && typeof document !== 'undefined') {
            setTimeout(() => {
                const newModal = document.querySelector(".modal-container");
                if (newModal && resizeObserver) {
                    modalContainer = newModal as HTMLElement;
                    resizeObserver.observe(modalContainer);
                    setLogoPosition();
                }
            }, 0);
        }
    }

    $: {
        if ($currentPage === "error") {
            pageComponent = NotFound;
        } else {
            const routeKey = $currentPage.toLowerCase();
            pageComponent = routes[routeKey] || NotFound;
            !routes[routeKey] && errorCode.set(404);
        }
    }
</script>

<svelte:head>
    <title>{AppName}</title>
</svelte:head>

<div class="app" class:dark={$theme === "dark"}>
    <header>
        <img src={logoPath} alt="Logo" class="logo" />
    </header>

    <button class="theme-toggle" on:click={toggleTheme} aria-label="Toggle theme">
        {#if $theme === "dark"}
            <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#FF3333" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="5" fill="#FF3333"></circle>
                <line x1="12" y1="1" x2="12" y2="3"></line>
                <line x1="12" y1="21" x2="12" y2="23"></line>
                <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                <line x1="1" y1="12" x2="3" y2="12"></line>
                <line x1="21" y1="12" x2="23" y2="12"></line>
                <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
            </svg>
        {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="#FF0000" stroke="#FF0000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
            </svg>
        {/if}
    </button>

    <main>
        <div class="modal">
            <svelte:component this={pageComponent} 
                            AppName={AppName} 
                            errorCode={$errorCode}
                            Fingerprint={$Fingerprint}/>
        </div>
    </main>
</div>

<style>
    :global(*.dark) {
        --white: #1e1e1e;
        --gray: #232323;
        --font-gray: #2a2a2a;
        --text-color: #f5f5f5;
        --gray-text-color: #a0a0a0;
    }

    .logo {
        position: fixed;
        height: 120px;
        width: auto;
        max-height: 15%;
        max-width: 380px;
        left: 50%;
        top: 50%;
    }

    .theme-toggle {
        position: fixed;
        top: 0;
        left: 20px;
        display: flex;
        justify-content: center;
        align-items: center;
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background-color: var(--white);
        box-shadow: var(--box-shadow);
        cursor: pointer;
        transition: transform 0.2s ease, box-shadow 0.2s ease;
        overflow: visible;
    }

    .theme-toggle:hover {
        transform: translateY(-2px) scale(1.05);
        box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    }

    .theme-toggle:active {
        transform: translateY(0) scale(0.95);
    }

    .theme-toggle svg {
        width: 28px;
        height: 28px;
        overflow: visible;
    }
</style>