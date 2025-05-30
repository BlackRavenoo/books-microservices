<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchAuthorDetails, fetchBooks } from '../api';
    import type { AuthorWithCover, BookPreview } from '../types';
    import { authStore } from '../store/authStore';
    import BookCard from '../components/BookCard.svelte';
    import { link } from 'svelte-routing';
    
    export let id: string;
    
    let author: AuthorWithCover | null = null;
    let books: BookPreview[] = [];
    let loading = true;
    let loadingBooks = true;
    let error = false;
    let booksError = false;

    let user = null;
    let isAdmin = false;
    
    let currentPage = 1;
    let pageSize = 24;
    let totalBooks = 0;
    let totalPages = 0;
    let sortOrder = "created_at";

    const sortOptions = [
        { value: "created_at", label: "Дата добавления" },
        { value: "chap_count", label: "Количество глав" },
        { value: "name_asc", label: "Название (А-Я)" },
        { value: "name_desc", label: "Название (Я-А)" }
    ];

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
        
        await loadBooks();
        
        return () => {
            unsubscribe();
        }
    });
    
    async function loadBooks() {
        loadingBooks = true;
        booksError = false;
        
        try {
            const response = await fetchBooks({
                target: "author",
                target_id: parseInt(id),
                page: currentPage,
                page_size: pageSize,
                order_by: sortOrder
            });
            
            books = response.items;
            totalBooks = response.total_items;
            totalPages = response.max_page;
        } catch (e) {
            booksError = true;
            books = [];
        } finally {
            loadingBooks = false;
        }
    }
    
    function changePage(newPage: number) {
        if (newPage >= 1 && newPage <= totalPages) {
            currentPage = newPage;
            loadBooks();
        }
    }
    
    function handleSortChange() {
        currentPage = 1;
        loadBooks();
    }
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
                        <a href={`/admin/edit-author/${author.id}`} use:link class="admin-icon-btn" title="Редактировать автора">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                        </a>
                    </div>
                {/if}
            </div>
            
            <div class="author-info">
                <h1 class="author-name">{author.name}</h1>
                
                <div class="author-stats">
                    <div class="stat-item">
                        <span class="stat-value">{totalBooks}</span>
                        <span class="stat-label">книг</span>
                    </div>
                </div>
            </div>
        </div>

        <div class="author-books">
            <div class="section-header">
                <h2 class="section-title">Книги автора</h2>
                
                <div class="sort-control">
                    <label for="sort-order">Сортировать по:</label>
                    <select id="sort-order" bind:value={sortOrder} on:change={handleSortChange}>
                        {#each sortOptions as option}
                            <option value={option.value}>{option.label}</option>
                        {/each}
                    </select>
                </div>
            </div>
            
            {#if loadingBooks}
                <div class="loading">Загрузка книг...</div>
            {:else if booksError}
                <div class="error">Не удалось загрузить книги автора</div>
            {:else if books.length === 0}
                <div class="books-placeholder">
                    У этого автора пока нет книг
                </div>
            {:else}
                <div class="books-grid">
                    {#each books as book (book.id)}
                        <BookCard {book} />
                    {/each}
                </div>
                
                {#if totalPages > 1}
                    <div class="pagination">
                        <button class="page-btn" disabled={currentPage === 1} on:click={() => changePage(1)}>
                            &laquo;
                        </button>
                        <button class="page-btn" disabled={currentPage === 1} on:click={() => changePage(currentPage - 1)}>
                            &lsaquo;
                        </button>
                        
                        {#each Array(totalPages > 5 ? 5 : totalPages) as _, i}
                            {@const pageNum = totalPages > 5 
                                ? Math.max(1, Math.min(currentPage - 2 + i, totalPages))
                                : i + 1}
                            {#if (pageNum >= currentPage - 2 && pageNum <= currentPage + 2) || 
                                  (totalPages <= 5) ||
                                  (currentPage <= 3 && pageNum <= 5) ||
                                  (currentPage >= totalPages - 2 && pageNum >= totalPages - 4)}
                                <button 
                                    class="page-btn" 
                                    class:active={currentPage === pageNum} 
                                    on:click={() => changePage(pageNum)}
                                >
                                    {pageNum}
                                </button>
                            {/if}
                        {/each}
                        
                        <button class="page-btn" disabled={currentPage === totalPages} on:click={() => changePage(currentPage + 1)}>
                            &rsaquo;
                        </button>
                        <button class="page-btn" disabled={currentPage === totalPages} on:click={() => changePage(totalPages)}>
                            &raquo;
                        </button>
                    </div>
                {/if}
            {/if}
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
    
    .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
    }
    
    .section-title {
        font-size: 1.5rem;
        font-weight: 700;
        margin: 0;
    }
    
    .sort-control {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    
    .sort-control select {
        padding: 0.5rem;
        border-radius: 4px;
        border: 1px solid var(--border-color);
        background-color: var(--light-bg);
        color: var(--text-primary);
    }
    
    .books-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 2rem;
    }
    
    .books-placeholder {
        padding: 2rem;
        text-align: center;
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        color: var(--text-muted);
    }
    
    .pagination {
        display: flex;
        justify-content: center;
        gap: 0.5rem;
        margin-top: 2rem;
    }
    
    .page-btn {
        width: 36px;
        height: 36px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
        border: 1px solid var(--border-color);
        background-color: var(--light-bg);
        color: var(--text-primary);
        cursor: pointer;
        transition: all 0.2s;
    }
    
    .page-btn:hover:not(:disabled) {
        background-color: var(--primary-color-light);
    }
    
    .page-btn.active {
        background-color: var(--primary-color);
        color: white;
    }
    
    .page-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
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
        
        .section-header {
            flex-direction: column;
            gap: 1rem;
        }
    }
</style>