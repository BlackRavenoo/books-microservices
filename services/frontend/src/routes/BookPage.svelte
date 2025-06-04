<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBookDetails, fetchBookChapters, rateBook, removeBookRating } from '../api';
    import type { Book, ChapterSchema } from '../types';
    import { authStore, type User } from '../store/authStore';
    import { bookStore, type BookState } from '../store/bookStore';
    import { link } from 'svelte-routing';
    
    export let id: string;

    let bookState: BookState;
    const unsubscribeBookStore = bookStore.subscribe(state => {
        bookState = state;
    });
    
    let book: Book | null = null;
    let chapters: ChapterSchema[] = [];
    let loading = true;
    let error = false;
    let activeTab = 'about';
    let userRating = 0;
    let showRatingModal = false;
    let selectedRating = 0;

    let user: User | null = null;
    let isAdmin = false;

    const unsubscribe = authStore.subscribe(state => {
        user = state.user;
        isAdmin = state.user?.roles?.includes('admin') || false;
    });
    
    onMount(async () => {
        try {
            console.log('BookPage onMount: bookState =', bookState);

            book = await fetchBookDetails(id);
            if (book) {
                if (book.rating?.user) {
                    userRating = book.rating.user;
                }
                bookStore.setBookData(book);
            }
        } catch (e) {
            error = true;
        } finally {
            loading = false;
        }
        
        return () => {
            unsubscribe();
            unsubscribeBookStore();
        }
    });

    async function handleTabChange(tab: string) {
        activeTab = tab;
        if (tab === 'chapters' && book?.chapters_count !== 0 && chapters.length === 0) {
            try {
                if (bookState.chapters.length > 0 && String(bookState.currentBook?.id) === id) {
                    chapters = bookState.chapters;
                } else {
                    chapters = await fetchBookChapters(id);
                    bookStore.setChapters(chapters);
                }
            } catch (e) {
                console.error('Failed to load chapters:', e);
            }
        }
    }

    function openRatingModal() {
        if (!user) return;
        selectedRating = userRating || 0;
        showRatingModal = true;
    }

    function closeRatingModal() {
        showRatingModal = false;
        selectedRating = 0;
    }

    async function submitRating() {
        if (!user || selectedRating === 0) return;
        try {
            await rateBook(parseInt(id), selectedRating);
            userRating = selectedRating;
            if (book && book.rating) {
                book.rating.user = selectedRating;
            }
            closeRatingModal();
        } catch (e) {
            console.error('Failed to rate book:', e);
        }
    }

    async function removeRating() {
        if (!user || !userRating) return;
        try {
            await removeBookRating(parseInt(id));
            userRating = 0;
            if (book && book.rating) {
                book.rating.user = 0;
            }
            closeRatingModal();
        } catch (e) {
            console.error('Failed to remove rating:', e);
        }
    }

    function formatDate(dateString: string): string {
        return new Date(dateString).toLocaleDateString('ru-RU', {
            day: '2-digit',
            month: '2-digit',
            year: 'numeric'
        });
    }
</script>

<div class="container">
    {#if loading}
        <div class="loading">Загрузка...</div>
    {:else if error || !book}
        <div class="error">Произошла ошибка при загрузке данных</div>
    {:else}
        <div class="book-layout">
            <div class="book-cover-section">
                <div class="book-cover">
                    <img src={book.cover} alt={book.title} />
                    
                    {#if isAdmin}
                        <div class="admin-buttons">
                            <button class="admin-icon-btn" title="Сообщение модератору">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
                            </button>
                            
                            <a href={`/book/${book.id}/chapters`} use:link class="admin-icon-btn" title="Управление главами">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>
                            </a>
                            
                            <a href={`/book/${book.id}/chapters/new`} use:link class="admin-icon-btn" title="Добавить главу">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><line x1="12" y1="8" x2="12" y2="16"></line><line x1="8" y1="12" x2="16" y2="12"></line></svg>
                            </a>
                            
                            <a href={`/admin/edit-book/${book.id}`} use:link class="admin-icon-btn" title="Редактирование книги">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg>
                            </a>
                        </div>
                    {/if}
                </div>
                
                <button class="read-button" class:disabled={book.chapters_count === 0}>
                    <div class="read-button-content">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path>
                            <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path>
                        </svg>
                        <span>{book.chapters_count > 0 ? 'Начать читать' : 'Нет глав'}</span>
                    </div>
                    {#if book.chapters_count > 0}
                        <span class="reading-progress">0/{book.chapters_count}</span>
                    {/if}
                </button>
            </div>

            <div class="book-info-section">
                <div class="book-header-info">
                    <h1 class="book-title">{book.title}</h1>
                    
                    <div class="rating-section">
                        {#if book.rating}
                            <div class="rating-display">
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" class="rating-star">
                                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                                </svg>
                                <span class="rating-value">{book.rating.avg.toFixed(2)}</span>
                            </div>
                        {/if}
                        
                        {#if user}
                            <button class="rate-button" on:click={openRatingModal}>
                                {#if userRating}
                                    {userRating} - моя оценка
                                {:else}
                                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" class="button-star">
                                        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                                    </svg>
                                    Оценить
                                {/if}
                            </button>
                        {:else}
                            <div class="rate-button disabled">
                                Войдите для оценки
                            </div>
                        {/if}
                    </div>
                </div>

                <div class="info-tabs">
                    <button 
                        class="tab-button" 
                        class:active={activeTab === 'about'}
                        on:click={() => handleTabChange('about')}
                    >
                        О книге
                    </button>
                    <button 
                        class="tab-button" 
                        class:active={activeTab === 'chapters'}
                        on:click={() => handleTabChange('chapters')}
                    >
                        Главы
                    </button>
                </div>

                <div class="tab-content">
                    {#if activeTab === 'about'}
                        <div class="book-about">
                            <div class="description">
                                <p>{book.description}</p>
                            </div>
                            
                            <div class="book-meta">
                                <div class="meta-item">
                                    <span class="meta-label">Статус:</span>
                                    <span class="meta-value">{book.status.name}</span>
                                </div>
                        
                                <div class="meta-item">
                                    <span class="meta-label">Авторы:</span>
                                    <div class="meta-list">
                                        {#each book.authors as author}
                                            <a href="/author/{author.id}" use:link class="meta-tag">{author.name}</a>
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
                        
                                <div class="meta-item">
                                    <span class="meta-label">Жанры:</span>
                                    <div class="meta-list">
                                        {#each book.genres as genre}
                                            <span class="meta-tag">{genre.name}</span>
                                        {/each}
                                    </div>
                                </div>
                            </div>
                        </div>
                    {:else if activeTab === 'chapters'}
                        <div class="chapters-content">
                            {#if chapters.length > 0}
                                <div class="chapters-list">
                                    {#each chapters as chapter}
                                        <div class="chapter-item">
                                            <a href={`/book/${book.id}/chapter?number=${chapter.index}`} use:link class="chapter-link">
                                                <span class="chapter-info">
                                                    <span class="chapter-number">Глава {chapter.index}</span>
                                                    <span class="chapter-name">{chapter.name}</span>
                                                </span>
                                            </a>
                                            <span class="chapter-date">{formatDate(chapter.created_at)}</span>
                                        </div>
                                    {/each}
                                </div>
                            {:else}
                                <div class="no-chapters">Главы не найдены</div>
                            {/if}
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    {/if}
</div>

{#if showRatingModal}
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div class="modal-overlay" on:click={closeRatingModal}>
        <div class="rating-modal" on:click|stopPropagation>
            <h3>Оценить книгу</h3>
            <p class="modal-book-title">{book?.title}</p>
            
            <div class="rating-stars-input">
                {#each Array(10) as _, i}
                    <button 
                        class="star-btn"
                        class:active={selectedRating > i}
                        class:hover={selectedRating === 0}
                        on:click={() => selectedRating = i + 1}
                        on:mouseenter={() => selectedRating === 0 && (selectedRating = i + 1)}
                        on:mouseleave={() => selectedRating === userRating && (selectedRating = userRating)}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                        </svg>
                    </button>
                {/each}
            </div>
            
            <div class="rating-value-display">
                {selectedRating}/10
            </div>
            
            <div class="modal-actions">
                <button class="cancel-btn" on:click={closeRatingModal}>
                    Отмена
                </button>
                
                {#if userRating}
                    <button 
                        class="remove-btn" 
                        on:click={removeRating}
                    >
                        Удалить оценку
                    </button>
                {/if}
                
                <button 
                    class="submit-btn" 
                    disabled={selectedRating === 0}
                    on:click={submitRating}
                >
                    {userRating ? 'Изменить' : 'Оценить'}
                </button>
            </div>
        </div>
    </div>
{/if}

<style>
    .container {
        width: 100%;
        max-width: 1200px;
        margin: 0 auto;
        padding: 2rem 1rem;
        background-color: var(--dark-bg);
        color: var(--text-light);
        min-height: 100vh;
    }
    
    .loading, .error {
        text-align: center;
        padding: 2rem;
        color: var(--text-muted);
    }
    
    .error {
        color: #ef4444;
    }
    
    .book-layout {
        display: flex;
        gap: 3rem;
        align-items: flex-start;
    }
    
    .book-cover-section {
        flex-shrink: 0;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.5rem;
    }
    
    .book-cover {
        position: relative;
        width: 280px;
    }
    
    .book-cover img {
        width: 100%;
        border-radius: 0.5rem;
        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.15);
    }
    
    .read-button {
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 0.5rem;
        padding: 1rem 2rem;
        font-size: 1.1rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        gap: 0.75rem;
        width: 100%;
        justify-content: center;
    }
    
    .read-button:hover {
        background-color: var(--secondary-color);
        transform: translateY(-2px);
        box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
    }
    
    .book-info-section {
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }
    
    .book-header-info {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        gap: 2rem;
    }
    
    .book-title {
        font-size: 2.5rem;
        font-weight: 700;
        margin: 0;
        flex: 1;
        line-height: 1.2;
    }
    
    .rating-section {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 0.75rem;
        flex-shrink: 0;
    }
    
    .rating-display {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 2rem;
        font-weight: 700;
        color: var(--primary-color);
        text-align: center;
        min-width: 60px;
    }

    .rating-star {
        color: #fbbf24;
        flex-shrink: 0;
    }
    
    .rate-button {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        background-color: var(--secondary-color);
        color: white;
        border: none;
        border-radius: 0.375rem;
        padding: 0.5rem 1rem;
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        transition: background-color 0.2s;
        white-space: nowrap;
    }

    .button-star {
        flex-shrink: 0;
    }
    
    .rate-button:hover {
        background-color: var(--primary-color);
    }
    
    .rate-button.disabled {
        background-color: var(--text-muted);
        cursor: not-allowed;
        font-size: 0.8rem;
    }
    
    .info-tabs {
        display: flex;
        border-bottom: 2px solid var(--light-bg);
    }
    
    .tab-button {
        background: none;
        border: none;
        padding: 1rem 1.5rem;
        cursor: pointer;
        font-size: 1.1rem;
        font-weight: 500;
        color: var(--text-muted);
        border-bottom: 3px solid transparent;
        transition: all 0.2s;
    }
    
    .tab-button.active {
        color: var(--primary-color);
        border-bottom-color: var(--primary-color);
    }
    
    .tab-button:hover {
        color: var(--primary-color);
    }
    
    .tab-content {
        flex: 1;
    }
    
    .book-about {
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }
    
    .description p {
        line-height: 1.7;
        font-size: 1.1rem;
        margin: 0;
        white-space: pre-line;
    }
    
    .book-meta {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
    
    .meta-item {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    
    .meta-label {
        font-weight: 600;
        color: var(--text-muted);
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    .meta-value {
        font-weight: 500;
    }
    
    .meta-list {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
    }
    
    .meta-tag {
        background-color: var(--light-bg);
        color: var(--text-light);
        padding: 0.375rem 0.75rem;
        border-radius: 1rem;
        font-size: 0.875rem;
        text-decoration: none;
        transition: all 0.2s ease;
    }
    
    a.meta-tag:hover {
        background-color: var(--primary-color);
        color: white;
        transform: translateY(-2px);
        box-shadow: 0 3px 5px rgba(0, 0, 0, 0.1);
    }
    
    .chapters-content {
        padding-top: 1rem;
    }
    
    .chapters-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    
    .chapter-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        background-color: var(--light-bg);
        border-radius: 0.5rem;
        transition: all 0.2s;
    }
    
    .chapter-item:hover {
        background-color: var(--primary-color);
        color: white;
        transform: translateX(5px);
    }
    
    .chapter-link {
        text-decoration: none;
        color: inherit;
        flex: 1;
    }
    
    .chapter-info {
        display: flex;
        align-items: center;
        gap: 1rem;
    }
    
    .chapter-number {
        font-weight: 600;
        min-width: 80px;
    }
    
    .chapter-name {
        flex: 1;
    }
    
    .chapter-date {
        font-size: 0.875rem;
        color: var(--text-muted);
        flex-shrink: 0;
    }
    
    .chapter-item:hover .chapter-date {
        color: rgba(255, 255, 255, 0.8);
    }
    
    .no-chapters {
        text-align: center;
        padding: 3rem;
        color: var(--text-muted);
        font-size: 1.1rem;
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
        text-decoration: none;
    }
    
    .admin-icon-btn:hover {
        background-color: rgba(255, 255, 255, 0.2);
    }
    
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(0, 0, 0, 0.7);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }
    
    .rating-modal {
        background-color: var(--light-bg);
        border-radius: 0.75rem;
        padding: 2rem;
        width: 90%;
        max-width: 500px;
        box-shadow: 0 20px 25px rgba(0, 0, 0, 0.3);
        color: var(--text-light);
    }
    
    .rating-modal h3 {
        margin: 0 0 0.5rem 0;
        font-size: 1.5rem;
        text-align: center;
        color: var(--text-light);
    }
    
    .modal-book-title {
        text-align: center;
        color: var(--text-muted);
        margin: 0 0 2rem 0;
        font-style: italic;
    }
    
    .rating-stars-input {
        display: flex;
        justify-content: center;
        gap: 0.25rem;
        margin: 2rem 0 1rem 0;
    }
    
    .star-btn {
        background: none;
        border: none;
        cursor: pointer;
        padding: 0.25rem;
        transition: all 0.2s ease;
        border-radius: 0.25rem;
    }
    
    .star-btn:hover {
        transform: scale(1.1);
    }
    
    .star-btn svg {
        width: 32px;
        height: 32px;
        color: #4a5568;
        transition: color 0.2s ease;
    }
    
    .star-btn.active svg {
        color: #fbbf24;
    }
    
    .rating-value-display {
        text-align: center;
        font-size: 1.25rem;
        font-weight: 600;
        color: var(--primary-color);
        margin-bottom: 2rem;
    }
    
    .modal-actions {
        display: flex;
        justify-content: center;
        gap: 1rem;
        flex-wrap: wrap;
    }
    
    .cancel-btn, .submit-btn, .remove-btn {
        padding: 0.75rem 1.5rem;
        border-radius: 0.5rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        border: none;
    }
    
    .cancel-btn {
        background-color: var(--text-muted);
        color: white;
    }
    
    .cancel-btn:hover {
        background-color: #4a5568;
    }
    
    .remove-btn {
        background-color: #dc2626;
        color: white;
    }
    
    .remove-btn:hover {
        background-color: #b91c1c;
    }
    
    .submit-btn {
        background-color: var(--primary-color);
        color: white;
    }
    
    .submit-btn:hover:not(:disabled) {
        background-color: var(--secondary-color);
    }
    
    .submit-btn:disabled {
        background-color: var(--text-muted);
        cursor: not-allowed;
        opacity: 0.5;
    }

    .read-button {
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 0.5rem;
        padding: 1rem 1.5rem;
        font-size: 1.1rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
    }
    
    .read-button:hover:not(.disabled) {
        background-color: var(--secondary-color);
        transform: translateY(-2px);
        box-shadow: 0 5px 15px rgba(0, 0, 0, 0.2);
    }
    
    .read-button.disabled {
        background-color: var(--text-muted);
        cursor: not-allowed;
        justify-content: center;
    }
    
    .read-button.disabled:hover {
        transform: none;
        box-shadow: none;
    }
    
    .read-button-content {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }
    
    .reading-progress {
        font-size: 0.95rem;
        font-weight: 500;
        opacity: 0.9;
    }
    
    @media (max-width: 768px) {
        .book-layout {
            flex-direction: column;
            gap: 2rem;
        }
        
        .book-cover {
            width: 240px;
        }
        
        .book-header-info {
            flex-direction: column;
            gap: 1rem;
        }
        
        .rating-section {
            align-self: flex-end;
        }
        
        .book-title {
            font-size: 2rem;
        }
        
        .rating-stars-input {
            gap: 0.15rem;
        }
        
        .star-btn svg {
            width: 28px;
            height: 28px;
        }
        
        .modal-actions {
            flex-direction: column;
        }
    }
</style>