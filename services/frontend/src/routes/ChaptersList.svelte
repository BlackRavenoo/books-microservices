<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBookChapters, deleteChapter } from '../api';
    import { link } from 'svelte-routing';
    
    export let bookId: string;
    
    let chapters: any[] = [];
    let isLoading = true;
    let error: string | null = null;
    
    onMount(async () => {
        await loadChapters();
    });
    
    async function loadChapters() {
        try {
            isLoading = true;
            chapters = await fetchBookChapters(bookId);
        } catch (err) {
            error = 'Не удалось загрузить главы';
            console.error(err);
        } finally {
            isLoading = false;
        }
    }
    
    async function handleDeleteChapter(chapterIndex: number, chapterName: string) {
        if (!confirm(`Вы уверены, что хотите удалить главу "${chapterName}"?`)) {
            return;
        }
        
        try {
            await deleteChapter(bookId, chapterIndex);
            await loadChapters();
        } catch (err) {
            alert('Не удалось удалить главу');
            console.error(err);
        }
    }
</script>

<div class="chapters-container">
    <div class="header">
        <h1>Главы книги</h1>
        <a href="/book/{bookId}/chapters/new" use:link class="add-chapter-btn">
            Добавить главу
        </a>
    </div>
    
    {#if error}
        <div class="error">{error}</div>
    {/if}
    
    {#if isLoading}
        <div class="loading">Загрузка глав...</div>
    {:else if chapters.length === 0}
        <div class="empty">
            <p>Глав пока нет</p>
            <a href="/book/{bookId}/chapters/new" use:link class="add-first-chapter">
                Создать первую главу
            </a>
        </div>
    {:else}
        <div class="chapters-list">
            {#each chapters as chapter}
                <div class="chapter-item">
                    <div class="chapter-info">
                        <span class="chapter-index">Глава {chapter.index}</span>
                        <h3 class="chapter-name">{chapter.name}</h3>
                        <span class="chapter-date">
                            {new Date(chapter.created_at).toLocaleDateString()}
                        </span>
                    </div>
                    
                    <div class="chapter-actions">
                        <a href="/book/{bookId}/chapter?number={chapter.index}" use:link class="read-btn">
                            Читать
                        </a>
                        <a href="/book/{bookId}/chapters/{chapter.index}/edit" use:link class="edit-btn">
                            Редактировать
                        </a>
                        <button 
                            class="delete-btn"
                            on:click={() => handleDeleteChapter(chapter.index, chapter.name)}
                        >
                            Удалить
                        </button>
                    </div>
                </div>
            {/each}
        </div>
    {/if}
    
    <div class="back-link">
        <a href="/book/{bookId}" use:link>← Вернуться к книге</a>
    </div>
</div>

<style>
    .chapters-container {
        max-width: 1000px;
        margin: 0 auto;
        padding: 2rem;
    }
    
    .header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 2rem;
    }
    
    .add-chapter-btn {
        background: var(--primary-color);
        color: white;
        padding: 0.75rem 1.5rem;
        text-decoration: none;
        border-radius: 4px;
        transition: background 0.2s;
    }
    
    .add-chapter-btn:hover {
        background: var(--secondary-color);
    }
    
    .error {
        background: #fee;
        color: #c33;
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
    }
    
    .loading {
        text-align: center;
        padding: 2rem;
        color: var(--text-muted);
    }
    
    .empty {
        text-align: center;
        padding: 3rem;
        color: var(--text-muted);
    }
    
    .add-first-chapter {
        background: var(--primary-color);
        color: white;
        padding: 1rem 2rem;
        text-decoration: none;
        border-radius: 4px;
        display: inline-block;
        margin-top: 1rem;
    }
    
    .chapters-list {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
    
    .chapter-item {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1.5rem;
        background: var(--light-bg);
        border-radius: 8px;
        border: 1px solid var(--border-color);
    }
    
    .chapter-info {
        flex: 1;
    }
    
    .chapter-index {
        font-size: 0.9rem;
        color: var(--text-muted);
        font-weight: 600;
    }
    
    .chapter-name {
        margin: 0.5rem 0;
        font-size: 1.25rem;
        color: var(--text-light);
    }
    
    .chapter-date {
        font-size: 0.9rem;
        color: var(--text-muted);
    }
    
    .chapter-actions {
        display: flex;
        gap: 0.75rem;
    }
    
    .read-btn, .edit-btn {
        padding: 0.5rem 1rem;
        text-decoration: none;
        border-radius: 4px;
        font-size: 0.9rem;
        transition: background 0.2s;
    }
    
    .read-btn {
        background: var(--primary-color);
        color: white;
    }
    
    .read-btn:hover {
        background: var(--secondary-color);
    }
    
    .edit-btn {
        background: var(--light-bg);
        color: var(--text-light);
        border: 1px solid var(--border-color);
    }
    
    .edit-btn:hover {
        background: var(--border-color);
    }
    
    .delete-btn {
        padding: 0.5rem 1rem;
        background: #dc3545;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 0.9rem;
        transition: background 0.2s;
    }
    
    .delete-btn:hover {
        background: #c82333;
    }
    
    .back-link {
        margin-top: 2rem;
        text-align: center;
    }
    
    .back-link a {
        color: var(--primary-color);
        text-decoration: none;
    }
    
    .back-link a:hover {
        text-decoration: underline;
    }
</style>