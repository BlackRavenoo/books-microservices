<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBookDetails } from '../api';
    import type { Book } from '../types';
    import { authStore } from '../store/authStore';
    
    export let id: string;
    
    let book: Book | null = null;
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
            book = await fetchBookDetails(id);
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
    {:else if error || !book}
        <div class="error">Произошла ошибка при загрузке данных</div>
    {:else}
        <div class="book-header">
            <div class="book-cover">
                <img src={book.cover} alt={book.title} />

                {#if isAdmin}
                    <div class="admin-buttons">
                        <button class="admin-icon-btn" title="Сообщение модератору">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
                        </button>
                        
                        <button class="admin-icon-btn" title="Редактирование глав">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
                        </button>
                        
                        <button class="admin-icon-btn" title="Добавить главы">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="12" y1="8" x2="12" y2="16"></line><line x1="8" y1="12" x2="16" y2="12"></line></svg>
                        </button>
                        
                        <a href={`/admin/edit-book/${book.id}`} class="admin-icon-btn" title="Редактирование тайтла">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                        </a>
                    </div>
                {/if}
            </div>
            <div class="book-info">
                <h1 class="book-title">{book.title}</h1>
          
                <div class="book-meta">
                    <div class="meta-item">
                        <span class="meta-label">Статус:</span>
                        <span class="meta-value">{book.status.name}</span>
                    </div>
            
                    <div class="meta-item">
                        <span class="meta-label">Авторы:</span>
                        <div class="meta-list">
                            {#each book.authors as author}
                                <a href="/author/{author.id}" class="meta-tag">{author.name}</a>
                            {/each}
                        </div>
                    </div>
            
                    <div class="meta-item">
                        <span class="meta-label">Жанры:</span>
                        <div class="meta-list">
                            {#each book.genres as genre}
                                <span class="meta-tag">{genre.name}</span>
                            {/each}
                        </div>
                    </div>
            
                    <div class="meta-item">
                        <span class="meta-label">Теги:</span>
                        <div class="meta-list">
                            {#each book.tags as tag}
                                <span class="meta-tag">{tag.name}</span>
                            {/each}
                        </div>
                    </div>
                </div>
          
                <div class="book-chapters">
                    <span class="chapters-count">{book.chapters_count} глав</span>
                    <button class="read-button">Читать</button>
                </div>
            </div>
        </div>
      
        <div class="book-description">
            <h2 class="section-title">Описание</h2>
            <div class="description-text">
                {book.description}
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
    
    .book-header {
        display: flex;
        gap: 2rem;
        margin-bottom: 2rem;
    }
    
    .book-cover {
        position: relative;
        flex-shrink: 0;
        width: 240px;
        align-self: flex-start;
    }
    
    .book-cover img {
        width: 100%;
        border-radius: 0.5rem;
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
    }
    
    .book-info {
        flex: 1;
    }
    
    .book-title {
        font-size: 2rem;
        margin-bottom: 1rem;
        font-weight: 700;
    }
    
    .book-meta {
        margin-bottom: 1.5rem;
    }
    
    .meta-item {
        margin-bottom: 0.75rem;
    }
    
    .meta-label {
        font-weight: 600;
        margin-right: 0.5rem;
        color: var(--text-muted);
    }
    
    .meta-list {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.5rem;
    }
    
    .meta-tag {
        background-color: var(--light-bg);
        color: var(--text-light);
        padding: 0.25rem 0.75rem;
        border-radius: 1rem;
        font-size: 0.875rem;
        text-decoration: none;
        transition: all 0.2s ease;
    }
    
    a.meta-tag:hover {
        background-color: var(--primary-color);
        color: white;
        cursor: pointer;
        transform: translateY(-2px);
        box-shadow: 0 3px 5px rgba(0, 0, 0, 0.1);
    }
    
    .book-chapters {
        display: flex;
        align-items: center;
        gap: 1rem;
        margin-top: 1.5rem;
    }
    
    .chapters-count {
        font-size: 1.125rem;
        font-weight: 500;
    }
    
    .read-button {
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 0.5rem;
        padding: 0.75rem 2rem;
        font-size: 1rem;
        font-weight: 600;
        cursor: pointer;
        transition: background-color 0.2s;
    }
    
    .read-button:hover {
        background-color: var(--secondary-color);
    }
    
    .book-description {
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        padding: 1.5rem;
    }
    
    .section-title {
        font-size: 1.25rem;
        font-weight: 700;
        margin-bottom: 1rem;
    }
    
    .description-text {
        line-height: 1.7;
        white-space: pre-line;
    }
    
    @media (max-width: 768px) {
        .book-header {
            flex-direction: column;
            align-items: center;
            text-align: center;
        }
      
        .book-cover {
            width: 180px;
            margin-bottom: 1.5rem;
        }
      
        .meta-list {
            justify-content: center;
        }
      
        .book-chapters {
            flex-direction: column;
        }
    }

    .admin-buttons {
        position: absolute;
        bottom: 7px;
        left: 0;
        right: 0;
        display: flex;
        justify-content: space-around;
        background: linear-gradient(to top,rgba(0,0,0,.9),transparent);
        padding-bottom: 6px;
        border-bottom-left-radius: 0.5rem;
        border-bottom-right-radius: 0.5rem;
    }

    .admin-icon-btn {
        background-color: transparent;
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
        background-color: rgba(255, 255, 255, 0.2);
    }
</style>