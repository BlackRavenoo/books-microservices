<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchBookDetails, fetchConstants, search, updateBook } from '../api';
    import type { UpdateBookFields } from '../types';
    import { link } from 'svelte-routing';

    export let id: number;
    
    let title = "";
    let description = "";
    let coverFile: File | null = null;
    let coverPreview: string | null = null;
    let originalCoverUrl: string | null = null;
    let selectedStatus: number | null = null;
    let selectedTags: number[] = [];
    let selectedGenres: number[] = [];
    let selectedAuthors: {id: number, name: string}[] = [];
    let seriesId: number | null = null;
    
    let originalTags: number[] = [];
    let originalGenres: number[] = [];
    let originalAuthors: number[] = [];
    
    let statuses: {id: number, name: string}[] = [];
    let tags: {id: number, name: string}[] = [];
    let genres: {id: number, name: string}[] = [];
    
    let authorSearchTerm = "";
    let authorSearchResults: {id: number, name: string}[] = [];
    let isSearchingAuthors = false;
    let showAuthorDropdown = false;
    
    let isLoading = true;
    let isSubmitting = false;
    let error: string | null = null;
    let success = false;
    
    function debounce<F extends (...args: any[]) => any, T = any>(func: F, wait: number): (...args: Parameters<F>) => void {
        let timeout: ReturnType<typeof setTimeout> | null = null;
        
        return function(this: T, ...args: Parameters<F>) {
            const context = this;
            
            if (timeout !== null) {
                clearTimeout(timeout);
            }
            
            timeout = setTimeout(() => {
                func.apply(context, args);
                timeout = null;
            }, wait);
        };
    }
    
    onMount(async () => {
        isLoading = true;
        try {
            const [book, constants] = await Promise.all([
                fetchBookDetails(id.toString()),
                fetchConstants()
            ]);
            
            if (!book) {
                throw new Error('Книга не найдена');
            }
            
            title = book.title;
            description = book.description;
            selectedStatus = book.status.id;
            originalCoverUrl = book.cover;
            coverPreview = book.cover;
            
            if (book.series_id) {
                seriesId = book.series_id;
            }
            
            originalTags = book.tags.map(t => t.id);
            originalGenres = book.genres.map(g => g.id);
            originalAuthors = book.authors.map(a => a.id);
            
            selectedTags = [...originalTags];
            selectedGenres = [...originalGenres];
            selectedAuthors = book.authors.map(a => ({ id: a.id, name: a.name }));
            
            statuses = constants.status;
            tags = constants.tags;
            genres = constants.genres;
            
        } catch (err: unknown) {
            error = err instanceof Error ? err.message : 'Ошибка при загрузке данных книги';
        } finally {
            isLoading = false;
        }
    });
    
    function handleCoverChange(event: Event) {
        const input = event.target as HTMLInputElement;
        if (!input.files || input.files.length === 0) {
            coverFile = null;
            coverPreview = originalCoverUrl;
            return;
        }
        
        coverFile = input.files[0];
        
        const reader = new FileReader();
        reader.onload = e => {
            coverPreview = e.target?.result as string;
        };
        reader.readAsDataURL(coverFile);
    }
    
    const searchAuthors = debounce(async (term: string) => {
        if (term.length < 2) {
            authorSearchResults = [];
            return;
        }
        
        isSearchingAuthors = true;
        try {
            authorSearchResults = await search(term, 'authors');
            showAuthorDropdown = true;
        } catch (err) {
            console.error('Author search failed:', err);
            authorSearchResults = [];
        } finally {
            isSearchingAuthors = false;
        }
    }, 300);
    
    function handleAuthorSearch() {
        showAuthorDropdown = true;
        searchAuthors(authorSearchTerm);
    }
    
    function selectAuthor(author: {id: number, name: string}) {
        if (!selectedAuthors.some(a => a.id === author.id)) {
            selectedAuthors = [...selectedAuthors, author];
        }
        authorSearchTerm = '';
        showAuthorDropdown = false;
        authorSearchResults = [];
    }
    
    function removeAuthor(authorId: number) {
        selectedAuthors = selectedAuthors.filter(a => a.id !== authorId);
    }
    
    function handleAuthorResultKeydown(event: KeyboardEvent, author: {id: number, name: string}) {
        if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            selectAuthor(author);
        }
    }
    
    async function handleSubmit() {
        if (!title) {
            error = 'Название книги обязательно';
            return;
        }
        
        if (!description) {
            error = 'Описание книги обязательно';
            return;
        }
        
        if (selectedStatus === null) {
            error = 'Выберите статус книги';
            return;
        }
        
        error = null;
        isSubmitting = true;
        
        try {
            const tagsToAdd = selectedTags.filter(id => !originalTags.includes(id));
            const tagsToDelete = originalTags.filter(id => !selectedTags.includes(id));
            
            const genresToAdd = selectedGenres.filter(id => !originalGenres.includes(id));
            const genresToDelete = originalGenres.filter(id => !selectedGenres.includes(id));
            
            const selectedAuthorIds = selectedAuthors.map(a => a.id);
            const authorsToAdd = selectedAuthorIds.filter(id => !originalAuthors.includes(id));
            const authorsToDelete = originalAuthors.filter(id => !selectedAuthorIds.includes(id));
            
            const fields: UpdateBookFields = {
                title,
                description,
                status: selectedStatus,
                tags_to_add: tagsToAdd,
                tags_to_delete: tagsToDelete,
                genres_to_add: genresToAdd,
                genres_to_delete: genresToDelete,
                authors_to_add: authorsToAdd,
                authors_to_delete: authorsToDelete
            };
            
            if (seriesId !== null) {
                fields.series_id = seriesId;
            }
            
            await updateBook(id, coverFile, fields);
            
            success = true;
            
        } catch (err: unknown) {
            error = err instanceof Error ? err.message : 'Произошла ошибка при обновлении книги';
        } finally {
            isSubmitting = false;
        }
    }
    
    function resetForm() {
        success = false;
        error = null;
        window.location.reload();
    }
</script>

<div class="container">
    <h1 class="form-title">Редактирование книги</h1>
    
    {#if isLoading}
        <div class="loading">Загрузка данных книги...</div>
    {:else if error}
        <div class="error-message">{error}</div>
    {:else if success}
        <div class="success-message">
            <h2>Книга успешно обновлена!</h2>
            <div class="action-buttons">
                <button class="action-button" on:click={resetForm}>Продолжить редактирование</button>
                <a href={`/book/${id}`} use:link class="action-button">Перейти к книге</a>
            </div>
        </div>
    {:else}
        <form on:submit|preventDefault={handleSubmit} class="book-form">
            <div class="form-group cover-upload">
                <label for="cover">Обложка книги</label>
                <div class="upload-container">
                    {#if coverPreview}
                        <div class="cover-preview">
                            <img src={coverPreview} alt="Preview" />
                            <button type="button" class="remove-cover" on:click={() => {
                                coverFile = null;
                                coverPreview = originalCoverUrl;
                            }}>×</button>
                        </div>
                    {:else}
                        <div class="upload-placeholder">
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M18 22H6c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h12c1.1 0 2 .9 2 2v16c0 1.1-.9 2-2 2m0-2V4H6v16h12M8 9V7h8v2H8m0 4v-2h8v2H8m0 4v-2h5v2H8Z"/></svg>
                            <span>Нажмите для выбора файла</span>
                        </div>
                    {/if}
                    <input 
                        type="file" 
                        id="cover" 
                        accept="image/*" 
                        on:change={handleCoverChange} 
                        class="file-input" 
                    />
                </div>
                <small>Рекомендуемый размер: 375px в ширину</small>
            </div>
            
            <div class="form-group">
                <label for="title">Название</label>
                <input 
                    type="text" 
                    id="title" 
                    bind:value={title} 
                    placeholder="Введите название книги" 
                    required
                />
            </div>
            
            <div class="form-group">
                <label for="description">Описание</label>
                <textarea 
                    id="description" 
                    bind:value={description} 
                    placeholder="Введите описание книги" 
                    rows="5" 
                    required
                ></textarea>
            </div>
            
            <div class="form-group">
                <label for="status">Статус</label>
                <select id="status" bind:value={selectedStatus}>
                    <option value={null}>Выберите статус</option>
                    {#each statuses as status}
                        <option value={status.id}>{status.name}</option>
                    {/each}
                </select>
            </div>
            
            <div class="form-group">
                <label for="tags-group" id="tags-label">Теги</label>
                <div id="tags-group" class="tags-container" aria-labelledby="tags-label" role="group">
                    {#each tags as tag}
                        <label class="tag-checkbox">
                            <input 
                                type="checkbox" 
                                value={tag.id} 
                                checked={selectedTags.includes(tag.id)}
                                on:change={(e) => {
                                    if (e.target.checked) {
                                        selectedTags = [...selectedTags, tag.id];
                                    } else {
                                        selectedTags = selectedTags.filter(id => id !== tag.id);
                                    }
                                }}
                            />
                            <span>{tag.name}</span>
                        </label>
                    {/each}
                </div>
            </div>
            
            <div class="form-group">
                <label for="genres-group" id="genres-label">Жанры</label>
                <div id="genres-group" class="tags-container" aria-labelledby="genres-label" role="group">
                    {#each genres as genre}
                        <label class="tag-checkbox">
                            <input 
                                type="checkbox" 
                                value={genre.id} 
                                checked={selectedGenres.includes(genre.id)}
                                on:change={(e) => {
                                    if (e.target.checked) {
                                        selectedGenres = [...selectedGenres, genre.id];
                                    } else {
                                        selectedGenres = selectedGenres.filter(id => id !== genre.id);
                                    }
                                }}
                            />
                            <span>{genre.name}</span>
                        </label>
                    {/each}
                </div>
            </div>
            
            <div class="form-group">
                <label for="author-search">Авторы</label>
                <div class="author-search">
                    <div class="search-input-container">
                        <input 
                            type="text" 
                            id="author-search"
                            placeholder="Начните вводить имя автора (минимум 2 символа)" 
                            bind:value={authorSearchTerm}
                            on:input={handleAuthorSearch}
                            on:focus={() => {
                                if (authorSearchTerm.length >= 2) {
                                    showAuthorDropdown = true;
                                }
                            }}
                        />
                        {#if isSearchingAuthors}
                            <div class="search-spinner"></div>
                        {/if}
                    </div>
                    
                    {#if showAuthorDropdown && authorSearchResults.length > 0}
                        <div class="author-dropdown" role="listbox">
                            {#each authorSearchResults as author}
                                <div 
                                    class="author-result"
                                    on:click={() => selectAuthor(author)}
                                    on:keydown={(e) => handleAuthorResultKeydown(e, author)}
                                    role="option"
                                    aria-selected="false"
                                    tabindex="0"
                                >
                                    {author.name}
                                </div>
                            {/each}
                        </div>
                    {/if}
                </div>
                
                {#if selectedAuthors.length > 0}
                    <div class="selected-authors">
                        {#each selectedAuthors as author}
                            <div class="selected-author">
                                <span>{author.name}</span>
                                <button 
                                    type="button" 
                                    class="remove-author"
                                    on:click={() => removeAuthor(author.id)}
                                    aria-label="Удалить автора {author.name}"
                                >×</button>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
            
            <div class="form-actions">
                <button 
                    type="submit" 
                    class="submit-button" 
                    disabled={isSubmitting}
                >
                    {isSubmitting ? 'Сохранение...' : 'Сохранить изменения'}
                </button>
            </div>
        </form>
    {/if}
</div>

<style>
    .container {
        width: 100%;
        max-width: 800px;
        margin: 0 auto;
        padding: 2rem 1rem;
    }
    
    .form-title {
        margin-bottom: 2rem;
        text-align: center;
        font-size: 1.75rem;
        font-weight: 700;
    }
    
    .loading, .error-message {
        text-align: center;
        padding: 2rem;
        border-radius: 0.5rem;
    }
    
    .loading {
        color: var(--text-muted);
    }
    
    .error-message {
        color: #ef4444;
        background-color: rgba(239, 68, 68, 0.1);
    }
    
    .success-message {
        color: #10b981;
        background-color: rgba(16, 185, 129, 0.1);
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.5rem;
        padding: 2rem;
        border-radius: 0.5rem;
    }

    .success-message h2 {
        margin: 0;
    }
    
    .action-buttons {
        display: flex;
        gap: 1rem;
        flex-wrap: wrap;
        justify-content: center;
    }
    
    .book-form {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }
    
    .form-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }
    
    label {
        font-weight: 600;
        color: var(--text-light);
    }
    
    input[type="text"], textarea, select {
        padding: 0.75rem;
        border: 1px solid var(--border-color);
        border-radius: 0.5rem;
        background-color: var(--light-bg);
        color: var(--text-light);
    }
    
    textarea {
        resize: vertical;
        min-height: 120px;
    }
    
    .cover-upload {
        margin-bottom: 1rem;
    }
    
    .upload-container {
        position: relative;
        width: 200px;
        height: 280px;
        border: 2px dashed var(--border-color);
        border-radius: 0.5rem;
        cursor: pointer;
        overflow: hidden;
    }
    
    .upload-placeholder {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: var(--text-muted);
        text-align: center;
        padding: 1rem;
    }
    
    .upload-placeholder svg {
        margin-bottom: 0.5rem;
        height: 48px;
        width: 48px;
    }
    
    .file-input {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        opacity: 0;
        cursor: pointer;
    }
    
    .cover-preview {
        position: relative;
        width: 100%;
        height: 100%;
    }
    
    .cover-preview img {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }
    
    .remove-cover {
        position: absolute;
        top: 5px;
        right: 5px;
        width: 24px;
        height: 24px;
        border-radius: 50%;
        background: rgba(0, 0, 0, 0.7);
        color: white;
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 16px;
    }
    
    .tags-container {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        max-height: 200px;
        overflow-y: auto;
        padding: 0.5rem;
        border: 1px solid var(--border-color);
        border-radius: 0.5rem;
    }
    
    .tag-checkbox {
        display: inline-flex;
        align-items: center;
        padding: 0.25rem 0.5rem;
        background-color: var(--light-bg);
        border-radius: 1rem;
        cursor: pointer;
        user-select: none;
    }
    
    .tag-checkbox input {
        margin-right: 0.25rem;
    }
    
    .author-search {
        position: relative;
    }
    
    .search-input-container {
        position: relative;
    }
    
    .search-spinner {
        position: absolute;
        right: 10px;
        top: 50%;
        transform: translateY(-50%);
        width: 16px;
        height: 16px;
        border: 2px solid rgba(0, 0, 0, 0.1);
        border-top-color: var(--primary-color);
        border-radius: 50%;
        animation: spin 1s infinite linear;
    }
    
    @keyframes spin {
        to { transform: translateY(-50%) rotate(360deg); }
    }
    
    .author-dropdown {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        background: var(--light-bg);
        border: 1px solid var(--border-color);
        border-radius: 0.5rem;
        margin-top: 0.25rem;
        max-height: 200px;
        overflow-y: auto;
        z-index: 10;
    }
    
    .author-result {
        padding: 0.75rem;
        cursor: pointer;
    }
    
    .author-result:hover {
        background-color: rgba(0, 0, 0, 0.05);
    }
    
    .author-result:focus {
        background-color: rgba(0, 0, 0, 0.1);
        outline: 2px solid var(--primary-color);
    }
    
    .selected-authors {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.5rem;
    }
    
    .selected-author {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.25rem 0.75rem;
        background-color: var(--primary-color);
        color: white;
        border-radius: 1rem;
    }
    
    .remove-author {
        background: none;
        border: none;
        color: white;
        font-size: 1rem;
        cursor: pointer;
        padding: 0;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .form-actions {
        margin-top: 1rem;
        display: flex;
        justify-content: center;
    }
    
    .action-button, .submit-button {
        padding: 0.75rem 2rem;
        background-color: var(--primary-color);
        color: white;
        border: none;
        border-radius: 0.5rem;
        font-weight: 600;
        cursor: pointer;
        transition: background-color 0.2s;
        text-decoration: none;
        display: inline-block;
        text-align: center;
    }
    
    .action-button:hover, .submit-button:hover {
        background-color: var(--secondary-color);
    }

    .action-button:focus, .submit-button:focus {
        outline: 2px solid var(--focus-color, #4299e1);
        outline-offset: 2px;
    }
    
    .submit-button:disabled {
        background-color: #ccc;
        cursor: not-allowed;
    }
</style>