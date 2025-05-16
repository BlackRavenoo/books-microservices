<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchAuthorDetails } from '../api';
    import type { AuthorWithCover } from '../types';
    import { authStore } from '../store/authStore';
    import { link } from 'svelte-routing';
    
    export let id: string;
    
    let author: AuthorWithCover | null = null;
    let loading = true;
    let error = false;

    let user = null;
    let isAdmin = false;

    const unsubscribe = authStore.subscribe(state => {
        user = state.user;
        isAdmin = state.user?.roles?.includes('admin') || false;
    });
    
    onMount(async () => {
        try {
            author = await fetchAuthorDetails(id);
        } catch (e) {
            error = true;
        } finally {
            loading = false;
        }
        
        return () => {
            unsubscribe();
        }
    });
</script>
  
<div class="container">
    {#if loading}
        <div class="loading">Загрузка...</div>
    {:else if error || !author}
        <div class="error">Произошла ошибка при загрузке данных автора</div>
    {:else}
        <div class="author-header">
            <div class="author-avatar-container">
                <img src={author.cover} alt={author.name} class="author-avatar" />
                
                {#if isAdmin}
                    <div class="admin-buttons">
                        <a href={`/admin/edit-author/${author.id}`} class="admin-icon-btn" title="Редактировать автора">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                        </a>
                    </div>
                {/if}
            </div>
            
            <div class="author-info">
                <h1 class="author-name">{author.name}</h1>
                
                <div class="author-stats">
                    <!-- This section can be expanded once API supports author stats -->
                    <div class="stat-item">
                        <span class="stat-value">0</span>
                        <span class="stat-label">книг</span>
                    </div>
                </div>
            </div>
        </div>
        
        <!-- Books by author section - placeholder for when this functionality is added -->
        <div class="author-books">
            <h2 class="section-title">Книги автора</h2>
            <div class="books-placeholder">
                Книги этого автора скоро появятся здесь
            </div>
        </div>
    {/if}
</div>
  
<style>
    .container {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 2rem 1rem;
    }
    
    .loading, .error {
        text-align: center;
        padding: 2rem;
        color: var(--text-muted);
    }
    
    .error {
        color: #ef4444;
    }
    
    .author-header {
        display: flex;
        align-items: center;
        gap: 2rem;
        margin-bottom: 3rem;
    }
    
    .author-avatar-container {
        position: relative;
        flex-shrink: 0;
    }
    
    .author-avatar {
        width: 180px;
        height: 180px;
        border-radius: 50%;
        object-fit: cover;
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    }
    
    .author-info {
        flex: 1;
    }
    
    .author-name {
        font-size: 2.5rem;
        margin-bottom: 1.5rem;
        font-weight: 700;
    }
    
    .author-stats {
        display: flex;
        gap: 2rem;
    }
    
    .stat-item {
        display: flex;
        flex-direction: column;
        align-items: center;
    }
    
    .stat-value {
        font-size: 1.5rem;
        font-weight: 700;
    }
    
    .stat-label {
        color: var(--text-muted);
        font-size: 0.9rem;
    }
    
    .admin-buttons {
        position: absolute;
        bottom: 0;
        right: 0;
        display: flex;
        gap: 0.5rem;
    }
    
    .admin-icon-btn {
        background-color: var(--primary-color);
        border: none;
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 32px;
        height: 32px;
        border-radius: 50%;
        cursor: pointer;
        transition: background-color 0.2s;
    }
    
    .admin-icon-btn:hover {
        background-color: var(--secondary-color);
    }
    
    .author-books {
        margin-top: 3rem;
    }
    
    .section-title {
        font-size: 1.5rem;
        font-weight: 700;
        margin-bottom: 1.5rem;
    }
    
    .books-placeholder {
        padding: 2rem;
        text-align: center;
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        color: var(--text-muted);
    }
    
    @media (max-width: 768px) {
        .author-header {
            flex-direction: column;
            text-align: center;
        }
        
        .author-avatar {
            margin-bottom: 1.5rem;
        }
        
        .author-stats {
            justify-content: center;
        }
    }
</style>