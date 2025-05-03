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
        {#if isAdmin}
            <div class="admin-controls">
                <h3 class="admin-heading">Управление книгой</h3>
                <div class="admin-buttons">
                    <a href={`/admin/edit-book/${book.id}`} class="admin-button edit-btn">Редактировать книгу</a>
                    <button class="admin-button delete-btn">Удалить книгу</button>
                </div>
            </div>
        {/if}
        <div class="book-header">
            <div class="book-cover">
                <img src={book.cover} alt={book.title} />
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
                                <span class="meta-tag">{author.name}</span>
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
        flex-shrink: 0;
        width: 240px;
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
</style>