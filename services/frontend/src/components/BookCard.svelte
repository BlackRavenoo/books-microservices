<script lang="ts">
    import { link } from 'svelte-routing';
    import type { BookPreview as BookPreview } from '../types';
    
    export let book: BookPreview;
    
    function getRatingColor(rating: number): string {
        if (rating === 0) return 'gray';
        if (rating >= 7) return 'green';
        if (rating >= 5) return 'yellow';
        return 'red';
    }
</script>
  
<a href={`/book/${book.id}`} use:link class="book-card">
    <div class="book-cover">
        <img src={book.thumbnail} alt="{book.title}" />
        {#if book.avg_rating !== undefined}
            <div class="rating-badge" class:green={getRatingColor(book.avg_rating) === 'green'} class:yellow={getRatingColor(book.avg_rating) === 'yellow'} class:red={getRatingColor(book.avg_rating) === 'red'} class:gray={getRatingColor(book.avg_rating) === 'gray'}>
                <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                </svg>
                <span>{book.avg_rating.toFixed(1)}</span>
            </div>
        {/if}
    </div>
    <div class="book-info">
        <h3 class="book-title">{book.title}</h3>
    </div>
</a>
  
<style>
    .book-card {
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        overflow: hidden;
        transition: transform 0.2s;
        text-decoration: none;
        color: var(--text-light);
        display: block;
    }
    
    .book-card:hover {
        transform: translateY(-5px);
    }
    
    .book-cover {
        position: relative;
        padding-top: 142%;
        background-color: #2d3748;
    }
    
    .book-cover img {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    
    .rating-badge {
        position: absolute;
        top: 12px;
        left: 8px;
        color: white;
        padding: 3px 6px;
        border-radius: 4px;
        font-size: 0.7rem;
        font-weight: 600;
        display: flex;
        align-items: center;
        gap: 2px;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }
    
    .rating-badge.green {
        background: linear-gradient(135deg, #10b981, #059669);
    }
    
    .rating-badge.yellow {
        background: linear-gradient(135deg, #fbbf24, #f59e0b);
    }
    
    .rating-badge.red {
        background: linear-gradient(135deg, #ef4444, #dc2626);
    }
    
    .rating-badge.gray {
        background: linear-gradient(135deg, #6b7280, #4b5563);
    }
    
    .book-info {
        padding: 0.75rem;
    }
    
    .book-title {
        font-size: 0.9rem;
        font-weight: 600;
        margin: 0;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
    }
</style>