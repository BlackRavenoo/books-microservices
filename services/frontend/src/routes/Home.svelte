<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBooks } from '../api';
    import type { BookPreview, BooksListPage } from '../types';
    import BookCard from '../components/BookCard.svelte';
    // @ts-ignore
    import Carousel from 'svelte-carousel';
    
    let page: BooksListPage | null = null;
    let books: BookPreview[] = [];
    let loading = true;
    let error = false;
    
    let particlesToShow = 5;
    
    function updateDisplayCount() {
        const width = window.innerWidth;
        if (width < 768) particlesToShow = 2;
        else if (width < 1024) particlesToShow = 3;
        else if (width < 1280) particlesToShow = 4;
        else particlesToShow = 5;
    }
    
    onMount(async () => {
        try {
            page = await fetchBooks({ 
                page_size: 20, 
                order_by: 'created_at'
            });
            books = page.items;
            loading = false;
            
            updateDisplayCount();
            window.addEventListener('resize', updateDisplayCount);
            
            return () => {
                window.removeEventListener('resize', updateDisplayCount);
            };
        } catch (e) {
            error = true;
            loading = false;
        }
    });
</script>

<div class="container">
    <div class="section-header">
        <h2 class="section-title">Новинки</h2>
    </div>
    
    {#if loading}
        <div class="loading">Загрузка...</div>
    {:else if error}
        <div class="error">Произошла ошибка при загрузке данных</div>
    {:else if books.length === 0}
        <div class="no-results">Книги не найдены</div>
    {:else}
        <div class="carousel-container">
            <Carousel
                particlesToShow={particlesToShow}
                particlesToScroll={particlesToShow}
                autoplay={false}
                arrows={true}
                dots={false}
                infinite={false}
                duration={500}
                swiping={true}
                let:loaded
            >
                {#each books as book (book.id)}
                    <div class="carousel-item">
                        <BookCard {book} />
                    </div>
                {/each}
            </Carousel>
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
    
    .carousel-container {
        position: relative;
        padding: 0;
    }

    .carousel-item {
        padding: 0 0.75rem;
        box-sizing: border-box;
    }
    
    .loading, .error, .no-results {
        text-align: center;
        padding: 2rem;
        color: var(--text-muted);
    }
    
    .error {
        color: #ef4444;
    }
    
    :global(.carousel-container .sc-carousel__arrow) {
        background-color: var(--light-bg);
        border: 1px solid var(--border-color);
        color: var(--text-light);
        width: 36px;
        height: 36px;
        border-radius: 50%;
    }
    
    :global(.carousel-container .sc-carousel__arrow:hover) {
        background-color: var(--border-color);
    }
    
    :global(.carousel-container .sc-carousel__arrow--disabled) {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>