<script lang="ts">
    import { link } from 'svelte-routing';
    import type { BookPreview as BookPreview } from '../types';
    
    export let book: BookPreview;
</script>
  
<a href={`/book/${book.id}`} use:link class="book-card">
    <div class="book-cover">
        <img src={book.thumbnail} alt="{book.title}" />
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