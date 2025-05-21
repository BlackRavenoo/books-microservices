<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBooks, fetchConstants } from '../api';
    import type { BookPreview, BooksListPage, Constants, } from '../types';
    import BookCard from '../components/BookCard.svelte';
    import { fly } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    
    let page: BooksListPage | null = null;
    let books: BookPreview[] = [];
    let constants: Constants | null = null;
    let loading = true;
    let loadingMore = false;
    let error = false;
    let currentPage = 1;
    let hasMorePages = true;
    
    let searchQuery = '';
    let selectedOrderBy = 'created_at';
    let selectedGenres: Record<number, -1 | 0 | 1> = {};
    let selectedTags: Record<number, -1 | 0 | 1> = {};
    let selectedStatuses: Record<number, boolean> = {};
    
    let activeFilterPanel: 'genres' | 'tags' | null = null;
    let debounceTimer: number;
    let isAnimatingBack = false;
    
    async function loadConstants() {
        try {
            constants = await fetchConstants();
            
            constants.status.forEach(status => {
                selectedStatuses[status.id] = true;
            });
        } catch (e) {
            console.error("Failed to load constants:", e);
        }
    }
    
    async function loadBooks(pageNum = 1, resetList = true) {
        try {
            if (pageNum === 1) {
                loading = true;
            } else {
                loadingMore = true;
            }
            
            const genresInclude = Object.entries(selectedGenres)
                .filter(([_, val]) => val === 1)
                .map(([id, _]) => parseInt(id));
                
            const genresExclude = Object.entries(selectedGenres)
                .filter(([_, val]) => val === -1)
                .map(([id, _]) => parseInt(id));
                
            const tagsInclude = Object.entries(selectedTags)
                .filter(([_, val]) => val === 1)
                .map(([id, _]) => parseInt(id));
                
            const tagsExclude = Object.entries(selectedTags)
                .filter(([_, val]) => val === -1)
                .map(([id, _]) => parseInt(id));
                
            const statuses = Object.entries(selectedStatuses)
                .filter(([_, selected]) => selected)
                .map(([id, _]) => parseInt(id));
            
            const result = await fetchBooks({ 
                page: pageNum,
                page_size: 60,
                order_by: selectedOrderBy,
                genres_include: genresInclude.length ? genresInclude : undefined,
                genres_exclude: genresExclude.length ? genresExclude : undefined,
                tags_include: tagsInclude.length ? tagsInclude : undefined,
                tags_exclude: tagsExclude.length ? tagsExclude : undefined,
                statuses: statuses.length ? statuses : undefined
            });
            
            if (pageNum === 1 || resetList) {
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
            loadBooks(currentPage + 1, false);
        }
    }
    
    function toggleFilterPanel(panel: 'genres' | 'tags') {
        isAnimatingBack = false;
        if (activeFilterPanel === panel) {
            activeFilterPanel = null;
        } else {
            activeFilterPanel = panel;
        }
    }
    
    function goBackToMainFilters() {
        isAnimatingBack = true;
        activeFilterPanel = null;
    }
    
    function toggleGenreFilter(id: number) {
        selectedGenres[id] = (selectedGenres[id] || 0) === 0 ? 1 : (selectedGenres[id] === 1 ? -1 : 0);
        applyFilters();
    }
    
    function toggleTagFilter(id: number) {
        selectedTags[id] = (selectedTags[id] || 0) === 0 ? 1 : (selectedTags[id] === 1 ? -1 : 0);
        applyFilters();
    }
    
    function toggleStatusFilter(id: number) {
        selectedStatuses[id] = !selectedStatuses[id];
        applyFilters();
    }
    
    function handleSortChange() {
        applyFilters();
    }
    
    function applyFilters() {
        loadBooks(1, true);
    }
    
    function clearFilters() {
        selectedGenres = {};
        selectedTags = {};
        
        if (constants) {
            constants.status.forEach(status => {
                selectedStatuses[status.id] = true;
            });
        }
        
        searchQuery = '';
        selectedOrderBy = 'created_at';
        applyFilters();
    }
    
    function handleKeydown(event: KeyboardEvent, callback: () => void) {
        if (event.key === 'Enter' || event.key === ' ') {
            callback();
        }
    }
    
    onMount(async () => {
        await loadConstants();
        await loadBooks();
        window.addEventListener('scroll', handleScroll);
        
        return () => {
            window.removeEventListener('scroll', handleScroll);
        };
    });
</script>

<div class="catalog-container">
    <div class="main-content">
        <h1 class="page-title">Каталог книг</h1>
        
        <div class="search-bar">
            <div class="search-input-wrapper disabled-search">
                <input 
                    type="text" 
                    placeholder="Поиск книг (пока не работает)..." 
                    bind:value={searchQuery}
                    disabled
                />
                <span class="search-icon">🔍</span>
            </div>
            
            <div class="sort-dropdown">
                <select bind:value={selectedOrderBy} on:change={handleSortChange}>
                    <option value="created_at">По дате добавления</option>
                    <option value="name_asc">По названию (А-Я)</option>
                    <option value="name_desc">По названию (Я-А)</option>
                    <option value="chap_count">По количеству глав</option>
                </select>
            </div>
        </div>
        
        {#if loading && books.length === 0}
            <div class="loading">Загрузка...</div>
        {:else if error && books.length === 0}
            <div class="error">Произошла ошибка при загрузке данных</div>
        {:else if books.length === 0}
            <div class="no-results">
                <p>Книги не найдены</p>
                <button class="clear-filters-btn" on:click={clearFilters}>Сбросить фильтры</button>
            </div>
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
    
    <div class="filters-panel">
        <div class="filters-content">
            <div class="panels-container">
                {#if !activeFilterPanel}
                    <div 
                        class="filter-main-view"
                        in:fly={{ x: isAnimatingBack ? -300 : 0, duration: 300, easing: quintOut }}
                        out:fly={{ x: -300, duration: 300, easing: quintOut }}
                    >
                        <h2>Фильтры</h2>
                        
                        {#if constants}
                            <div class="filter-section">
                                <h3>Статус</h3>
                                <div class="status-filters">
                                    {#each constants.status as status}
                                        <label class="status-checkbox">
                                            <input 
                                                type="checkbox" 
                                                checked={selectedStatuses[status.id]} 
                                                on:change={() => toggleStatusFilter(status.id)} 
                                            />
                                            <span>{status.name}</span>
                                        </label>
                                    {/each}
                                </div>
                            </div>
                            
                            <div class="filter-section">
                                <h3>Фильтры</h3>
                                <div class="filter-buttons">
                                    <button 
                                        class="filter-btn {activeFilterPanel === 'genres' ? 'active' : ''}" 
                                        on:click={() => toggleFilterPanel('genres')}
                                    >
                                        Жанры
                                        {#if Object.values(selectedGenres).some(v => v !== 0)}
                                            <span class="filter-badge"></span>
                                        {/if}
                                    </button>
                                    <button 
                                        class="filter-btn {activeFilterPanel === 'tags' ? 'active' : ''}" 
                                        on:click={() => toggleFilterPanel('tags')}
                                    >
                                        Теги
                                        {#if Object.values(selectedTags).some(v => v !== 0)}
                                            <span class="filter-badge"></span>
                                        {/if}
                                    </button>
                                </div>
                            </div>
                            
                            <button class="clear-filters-btn" on:click={clearFilters}>Сбросить фильтры</button>
                        {/if}
                    </div>
                {:else}
                    <div 
                        class="filter-panel-view" 
                        in:fly={{ x: 300, duration: 300, easing: quintOut }}
                        out:fly={{ x: 300, duration: 300, easing: quintOut }}
                    >
                        <div class="panel-header">
                            <button 
                                class="back-btn" 
                                on:click={goBackToMainFilters}
                                aria-label="Вернуться к фильтрам"
                            >
                                ←
                            </button>
                            <h3>{activeFilterPanel === 'genres' ? 'Жанры' : 'Теги'}</h3>
                        </div>
                        
                        <div class="panel-content">
                            {#if activeFilterPanel === 'genres' && constants}
                                {#each constants.genres as genre}
                                    <button 
                                        class="filter-item {selectedGenres[genre.id] === 1 ? 'included' : selectedGenres[genre.id] === -1 ? 'excluded' : ''}"
                                        on:click={() => toggleGenreFilter(genre.id)}
                                        on:keydown={(e) => handleKeydown(e, () => toggleGenreFilter(genre.id))}
                                        tabindex="0"
                                    >
                                        <span class="filter-checkbox">
                                            {#if selectedGenres[genre.id] === 1}
                                                ✓
                                            {:else if selectedGenres[genre.id] === -1}
                                                ✕
                                            {/if}
                                        </span>
                                        <span class="filter-name">{genre.name}</span>
                                    </button>
                                {/each}
                            {:else if activeFilterPanel === 'tags' && constants}
                                {#each constants.tags as tag}
                                    <button 
                                        class="filter-item {selectedTags[tag.id] === 1 ? 'included' : selectedTags[tag.id] === -1 ? 'excluded' : ''}"
                                        on:click={() => toggleTagFilter(tag.id)}
                                        on:keydown={(e) => handleKeydown(e, () => toggleTagFilter(tag.id))}
                                        tabindex="0"
                                    >
                                        <span class="filter-checkbox">
                                            {#if selectedTags[tag.id] === 1}
                                                ✓
                                            {:else if selectedTags[tag.id] === -1}
                                                ✕
                                            {/if}
                                        </span>
                                        <span class="filter-name">{tag.name}</span>
                                    </button>
                                {/each}
                            {/if}
                        </div>
                    </div>
                {/if}
            </div>
        </div>
    </div>
</div>

<style>
    .catalog-container {
        display: flex;
        width: 100%;
        max-width: 1400px;
        margin: 0 auto;
        position: relative;
    }
    
    .main-content {
        flex: 1;
        padding: 2rem 1rem;
        min-height: 100vh;
        border-right: 1px solid rgba(0, 0, 0, 0.1);
    }
    
    .page-title {
        font-size: 2rem;
        font-weight: 700;
        margin-bottom: 1.5rem;
    }
    
    .search-bar {
        display: flex;
        gap: 1rem;
        margin-bottom: 2rem;
        align-items: center;
    }
    
    .search-input-wrapper {
        flex: 1;
        position: relative;
    }
    
    .search-input-wrapper input {
        width: 100%;
        padding: 0.75rem 1rem 0.75rem 2.5rem;
        border: 1px solid rgba(0, 0, 0, 0.1);
        border-radius: 0.5rem;
        font-size: 1rem;
    }
    
    .search-icon {
        position: absolute;
        left: 0.75rem;
        top: 50%;
        transform: translateY(-50%);
        opacity: 0.5;
    }

    .disabled-search {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .disabled-search input {
        cursor: not-allowed;
        background-color: rgba(255, 255, 255, 0.1);
    }
    
    .sort-dropdown select {
        padding: 0.75rem 1rem;
        border: 1px solid rgba(0, 0, 0, 0.1);
        border-radius: 0.5rem;
        font-size: 1rem;
        background-color: var(--primary-color);
        color: white;
    }
    
    .books-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
        gap: 1.5rem;
    }
    
    .filters-panel {
        width: 280px;
        background-color: #2a2a2a;
        position: sticky;
        top: 0;
        height: 100vh;
        overflow: hidden;
        color: white;
    }
    
    .filters-content {
        padding: 2rem 1rem;
        position: relative;
        height: 100%;
        overflow-y: auto;
    }
    
    .panels-container {
        position: relative;
        min-height: 400px;
    }
    
    .filter-main-view, .filter-panel-view {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
    }
    
    .filters-panel h2 {
        font-size: 1.4rem;
        font-weight: 600;
        margin-bottom: 1.5rem;
        color: white;
    }
    
    .filter-section {
        margin-bottom: 2rem;
    }
    
    .filter-section h3 {
        font-size: 1.1rem;
        font-weight: 600;
        margin-bottom: 1rem;
        color: white;
    }
    
    .status-filters {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    
    .status-checkbox {
        display: flex;
        align-items: center;
        cursor: pointer;
        color: white;
    }
    
    .status-checkbox input {
        margin-right: 0.5rem;
    }
    
    .filter-buttons {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
    }
    
    .filter-btn {
        padding: 0.6rem 1rem;
        background-color: #3a3a3a;
        border: none;
        border-radius: 0.5rem;
        cursor: pointer;
        position: relative;
        font-weight: 500;
        color: white;
    }
    
    .filter-btn.active {
        background-color: var(--primary-color);
        color: white;
    }
    
    .filter-badge {
        position: absolute;
        top: -5px;
        right: -5px;
        width: 10px;
        height: 10px;
        background-color: #ef4444;
        border-radius: 50%;
    }
    
    .clear-filters-btn {
        padding: 0.6rem 1rem;
        background-color: rgba(255, 255, 255, 0.1);
        border: none;
        border-radius: 0.5rem;
        cursor: pointer;
        width: 100%;
        margin-top: 1rem;
        color: white;
    }
    
    .clear-filters-btn:hover {
        background-color: rgba(255, 255, 255, 0.2);
    }
    
    .panel-header {
        display: flex;
        align-items: center;
        margin-bottom: 1.5rem;
    }
    
    .panel-header h3 {
        font-size: 1.2rem;
        font-weight: 600;
        color: white;
        margin: 0;
    }
    
    .back-btn {
        background: none;
        border: none;
        font-size: 1.25rem;
        cursor: pointer;
        padding: 0.25rem 0.5rem;
        margin-right: 0.75rem;
        color: white;
        border-radius: 0.25rem;
    }
    
    .back-btn:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }
    
    .panel-content {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    
    .filter-item {
        display: flex;
        align-items: center;
        padding: 0.75rem;
        border-radius: 0.375rem;
        cursor: pointer;
        background: none;
        border: none;
        width: 100%;
        text-align: left;
        font-size: 1rem;
        color: white;
    }
    
    .filter-item:hover {
        background-color: rgba(255, 255, 255, 0.1);
    }
    
    .filter-item.included {
        background-color: rgba(16, 185, 129, 0.2);
    }
    
    .filter-item.excluded {
        background-color: rgba(239, 68, 68, 0.2);
    }
    
    .filter-checkbox {
        width: 24px;
        height: 24px;
        border: 1px solid rgba(255, 255, 255, 0.3);
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-right: 0.75rem;
        background-color: #3a3a3a;
    }
    
    .filter-item.included .filter-checkbox {
        background-color: #10b981;
        border-color: #10b981;
        color: white;
    }
    
    .filter-item.excluded .filter-checkbox {
        background-color: #ef4444;
        border-color: #ef4444;
        color: white;
    }
    
    .loading, .error, .no-results, .loading-more {
        text-align: center;
        padding: 2rem;
        opacity: 0.7;
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
    
    @media (max-width: 768px) {
        .catalog-container {
            flex-direction: column;
        }
        
        .filters-panel {
            width: 100%;
            height: auto;
            position: relative;
        }
    }
</style>