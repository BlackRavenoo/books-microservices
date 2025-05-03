<script lang="ts">
    import { searchBooks } from '../api';
    import type { BookSearchResult } from '../types';
    import { link } from 'svelte-routing';
    import { onDestroy, onMount } from 'svelte';
    import { authStore, type User } from '../store/authStore';
    import { startAuth } from '../utils/auth';
    
    let searchQuery = '';
    let searchResults: BookSearchResult[] = [];
    let showResults = false;
    let debounceTimeout: number;
    let showSearchBar = false;
    let searchType = 'books';
    let isClosing = false;
    let searchContainer: HTMLElement;

    let user: User | null = null;
    let isAdmin = false;

    const unsubscribe = authStore.subscribe(state => {
        user = state.user;
        isAdmin = state.user?.roles?.includes('admin') || false;
    });
    
    function handleLogin() {
        startAuth();
    }
    
    function handleLogout() {
        authStore.logout();
    }
    
    onDestroy(() => {
        unsubscribe();
    });
    
    onMount(() => {
        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    });
    
    function handleClickOutside(event: MouseEvent) {
        if (showSearchBar && searchContainer && !searchContainer.contains(event.target as Node)) {
            closeSearchBar();
        }
    }
    
    async function handleSearch() {
        if (searchQuery.length < 2) {
            searchResults = [];
            showResults = false;
            return;
        }
        
        clearTimeout(debounceTimeout);
        debounceTimeout = setTimeout(async () => {
            searchResults = await searchBooks(searchQuery);
            showResults = true;
        }, 300);
    }
    
    function toggleSearchBar() {
        if (showSearchBar) {
            closeSearchBar();
        } else {
            showSearchBar = true;
            setTimeout(() => {
                (document.querySelector('.search-input') as HTMLInputElement)?.focus();
            }, 100);
        }
    }
    
    function closeSearchBar() {
        isClosing = true;
        setTimeout(() => {
            showSearchBar = false;
            isClosing = false;
            searchQuery = '';
            searchResults = [];
            showResults = false;
        }, 300);
    }
</script>
  
<header>
    <div class="container">
        <a href="/" use:link class="logo">BooksLib</a>
        
        <nav>
            <a href="/catalog" use:link class="nav-link">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
                    <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
                </svg>
                Каталог
            </a>
            
            <button class="nav-link search-btn" on:click={toggleSearchBar}>
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="11" cy="11" r="8"></circle>
                    <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
                Поиск
            </button>
        </nav>
        
        <div class="auth-section">
            {#if user}
                <div class="user-menu">
                    <button class="user-button">
                        {#if user.avatar_url}
                            <img src={user.avatar_url} alt={user.username} class="avatar" />
                        {/if}
                        <span>{user.username}</span>
                    </button>
                    <div class="dropdown-menu">
                        <a href="/profile" use:link class="dropdown-item">Профиль</a>
                        <a href="/bookmarks" use:link class="dropdown-item">Закладки</a>
                        
                        {#if isAdmin}
                            <div class="dropdown-divider"></div>
                            <a href="/admin/books" use:link class="dropdown-item admin-item">Управление книгами</a>
                            <a href="/admin/users" use:link class="dropdown-item admin-item">Управление пользователями</a>
                        {/if}
                        
                        <div class="dropdown-divider"></div>
                        <button class="dropdown-item logout-btn" on:click={handleLogout}>Выйти</button>
                    </div>
                </div>
            {:else}
                <button class="auth-button" on:click={handleLogin}>Войти</button>
            {/if}
        </div>
    </div>
    
    {#if showSearchBar}
        <div class="search-overlay-wrapper">
            <div class="search-overlay" class:closing={isClosing}>
                <div class="search-container" bind:this={searchContainer}>
                    <div class="search-bar">
                        <input 
                            type="text" 
                            class="search-input" 
                            placeholder="Поиск..." 
                            bind:value={searchQuery}
                            on:input={handleSearch}
                        />
                    </div>
                    
                    <div class="search-types">
                        <button 
                            class="search-type-tab" 
                            class:active={searchType === 'books'}
                            on:click={() => searchType = 'books'}
                        >
                            Книги
                        </button>
                        <button 
                            class="search-type-tab" 
                            class:active={searchType === 'authors'}
                            on:click={() => searchType = 'authors'}
                        >
                            Авторы
                        </button>
                    </div>
                    
                    {#if showResults && searchResults.length > 0}
                        <div class="search-results">
                            {#each searchResults as result}
                                <a href={`/book/${result.id}`} use:link class="search-result-item" on:click={toggleSearchBar}>
                                    <img src={result.cover} alt={result.title} class="result-cover" />
                                    <div class="result-info">
                                        <div class="result-title">{result.title}</div>
                                        <div class="result-status">{result.status}</div>
                                    </div>
                                </a>
                            {/each}
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    {/if}
</header>
  
<style>
    header {
        background-color: var(--light-bg);
        border-bottom: 1px solid var(--border-color);
        padding: 1rem 0;
        position: relative;
    }
    
    .container {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 0 1rem;
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    
    .logo {
        font-size: 1.5rem;
        font-weight: 700;
        color: var(--primary-color);
        text-decoration: none;
        flex: 1;
    }
    
    nav {
        display: flex;
        gap: 1.5rem;
        flex: 1;
        justify-content: center;
    }
    
    .auth-section {
        flex: 1;
        display: flex;
        justify-content: flex-end;
        align-items: center;
    }

    .auth-button {
        padding: 0.5rem 1rem;
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 0.25rem;
        font-weight: 500;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .auth-button:hover {
        background-color: var(--secondary-color);
    }
    
    .nav-link {
        color: var(--text-light);
        text-decoration: none;
        font-weight: 500;
        transition: color 0.2s;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background: none;
        border: none;
        padding: 0;
        font-size: 1rem;
        cursor: pointer;
        font-family: inherit;
    }
    
    .nav-link:hover {
        color: var(--primary-color);
    }
    
    .search-overlay-wrapper {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        display: flex;
        justify-content: center;
        z-index: 100;
    }
    
    .search-overlay {
        width: 500px;
        background-color: var(--dark-bg);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        animation: slideDown 0.3s ease-out forwards;
        padding: 1rem;
    }
    
    .search-overlay.closing {
        animation: slideUp 0.3s ease-out forwards;
    }
    
    @keyframes slideDown {
        0% {
            transform: translateY(-100%);
            opacity: 0;
        }
        100% {
            transform: translateY(0);
            opacity: 1;
        }
    }
    
    @keyframes slideUp {
        0% {
            transform: translateY(0);
            opacity: 1;
        }
        100% {
            transform: translateY(-100%);
            opacity: 0;
        }
    }
    
    .search-container {
        width: 100%;
    }
    
    .search-bar {
        display: flex;
        align-items: center;
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        padding: 0.75rem 1rem;
        margin-bottom: 0.75rem;
    }
    
    .search-input {
        background: transparent;
        border: none;
        outline: none;
        color: var(--text-light);
        width: 100%;
        font-size: 1.1rem;
    }
    
    .search-types {
        display: flex;
        gap: 2rem;
        margin-bottom: 0.75rem;
        justify-content: flex-start;
    }
    
    .search-type-tab {
        background: none;
        border: none;
        color: var(--text-muted);
        font-size: 1rem;
        cursor: pointer;
        padding: 0.5rem 0;
        position: relative;
        transition: color 0.2s;
    }
    
    .search-type-tab.active {
        color: var(--text-light);
    }
    
    .search-type-tab.active::after {
        content: '';
        position: absolute;
        bottom: 0;
        left: 0;
        width: 100%;
        height: 2px;
        background-color: var(--primary-color);
    }
    
    .search-results {
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        margin-top: 0.5rem;
        max-height: 400px;
        overflow-y: auto;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    }
    
    .search-result-item {
        display: flex;
        padding: 0.75rem;
        text-decoration: none;
        color: var(--text-light);
        border-bottom: 1px solid var(--border-color);
    }
    
    .search-result-item:last-child {
        border-bottom: none;
    }
    
    .search-result-item:hover {
        background-color: rgba(255, 255, 255, 0.05);
    }
    
    .result-cover {
        width: 50px;
        height: 70px;
        object-fit: cover;
        border-radius: 0.25rem;
    }
    
    .result-info {
        margin-left: 0.75rem;
    }
    
    .result-title {
        font-weight: 500;
        margin-bottom: 0.25rem;
    }
    
    .result-status {
        font-size: 0.75rem;
        color: var(--text-muted);
    }
</style>