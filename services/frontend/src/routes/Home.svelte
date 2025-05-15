<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBooks } from '../api';
    import type { BookPreview, BooksListPage } from '../types';
    import BookCard from '../components/BookCard.svelte';
    
    let page: BooksListPage | null = null;
    let books: BookPreview[] = [];
    let loading = true;
    let error = false;
    
    onMount(async () => {
        try {
            page = await fetchBooks();
            books = page.items;
            loading = false;
        } catch (e) {
            error = true;
            loading = false;
        }
    });
</script>
  
<div class="container">
    <h2 class="section-title">Популярные книги</h2>
    
    {#if loading}
        <div class="loading">Загрузка...</div>
    {:else if error}
        <div class="error">Произошла ошибка при загрузке данных</div>
    {:else if books.length === 0}
        <div class="no-results">Книги не найдены</div>
    {:else}
        <div class="books-grid">
            {#each books as book (book.id)}
                <BookCard {book} />
            {/each}
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
    
    .section-title {
        font-size: 1.5rem;
        font-weight: 700;
        margin-bottom: 1.5rem;
    }
    
    .books-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1.5rem;
    }
    
    .loading, .error, .no-results {
        text-align: center;
        padding: 2rem;
        color: var(--text-muted);
    }
    
    .error {
        color: #ef4444;
    }
</style>