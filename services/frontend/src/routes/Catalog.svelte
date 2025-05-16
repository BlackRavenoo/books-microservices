<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBooks } from '../api';
    import type { BookPreview, BooksListPage } from '../types';
    import BookCard from '../components/BookCard.svelte';
    
    let page: BooksListPage | null = null;
    let books: BookPreview[] = [];
    let loading = true;
    let loadingMore = false;
    let error = false;
    let currentPage = 1;
    let hasMorePages = true;
    
    async function loadBooks(pageNum = 1) {
        try {
            if (pageNum === 1) {
                loading = true;
            } else {
                loadingMore = true;
            }
            
            const result = await fetchBooks({ 
                page: pageNum,
                page_size: 60,
                order_by: 'created_at'
            });
            
            if (pageNum === 1) {
                books = result.items;
                page = result;
            } else {
                books = [...books, ...result.items];
            }
            
            hasMorePages = result.max_page > pageNum;
            currentPage = pageNum;
            
        } catch (e) {
            error = pageNum === 1;
            console.error("Failed to load books:", e);
        } finally {
            loading = false;
            loadingMore = false;
        }
    }
    
    function handleScroll() {
        if (loadingMore || !hasMorePages) return;
        
        const scrollY = window.scrollY;
        const visible = document.documentElement.clientHeight;
        const pageHeight = document.documentElement.scrollHeight;
        
        const bottomOfPage = visible + scrollY >= pageHeight - 500;
        
        if (bottomOfPage) {
            loadBooks(currentPage + 1);
        }
    }
    
    onMount(async () => {
        await loadBooks();
        window.addEventListener('scroll', handleScroll);
        
        return () => {
            window.removeEventListener('scroll', handleScroll);
        };
    });
</script>

<div class="container">
    <h1 class="page-title">Каталог книг</h1>
    
    {#if loading && books.length === 0}
        <div class="loading">Загрузка...</div>
    {:else if error && books.length === 0}
        <div class="error">Произошла ошибка при загрузке данных</div>
    {:else if books.length === 0}
        <div class="no-results">Книги не найдены</div>
    {:else}
        <div class="books-grid">
            {#each books as book (book.id)}
                <BookCard {book} />
            {/each}
        </div>
        
        {#if loadingMore}
            <div class="loading-more">
                <div class="spinner"></div>
            </div>
        {/if}
    {/if}
</div>

<style>
    .container {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 2rem 1rem;
    }
    
    .page-title {
        font-size: 2rem;
        font-weight: 700;
        margin-bottom: 2rem;
    }
    
    .books-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1.5rem;
    }
    
    .loading, .error, .no-results, .loading-more {
        text-align: center;
        padding: 2rem;
        color: var(--text-muted);
    }
    
    .loading-more {
        padding: 1rem;
    }
    
    .error {
        color: #ef4444;
    }

    .loading-more {
        text-align: center;
        padding: 1.5rem;
        color: var(--text-muted);
        height: 80px;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .spinner {
        width: 30px;
        height: 30px;
        border: 3px solid rgba(0, 0, 0, 0.1);
        border-radius: 50%;
        border-top-color: var(--primary-color);
        animation: spin 1s ease-in-out infinite;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
</style>