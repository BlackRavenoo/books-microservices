<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchAuthorDetails, updateAuthor } from '../api';
    import { link } from 'svelte-routing';
    
    export let id: string;
    
    let name = "";
    let coverFile: File | null = null;
    let coverPreview: string | null = null;
    let originalCoverUrl: string | null = null;
    
    let isLoading = true;
    let isSubmitting = false;
    let error: string | null = null;
    let success = false;
    
    onMount(async () => {
        isLoading = true;
        try {
            const author = await fetchAuthorDetails(id.toString());
            if (!author) {
                throw new Error('Автор не найден');
            }
            
            name = author.name;
            originalCoverUrl = author.cover;
            coverPreview = author.cover;
        } catch (err: unknown) {
            error = err instanceof Error ? err.message : 'Ошибка при загрузке данных автора';
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
    
    async function handleSubmit() {
        if (!name) {
            error = 'Имя автора обязательно';
            return;
        }
        
        error = null;
        isSubmitting = true;
        
        try {
            await updateAuthor(id, coverFile, { name });
            success = true;
        } catch (err: unknown) {
            error = err instanceof Error ? err.message : 'Произошла ошибка при обновлении автора';
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
    <h1 class="form-title">Редактирование автора</h1>
    
    {#if isLoading}
        <div class="loading">Загрузка данных автора...</div>
    {:else if error}
        <div class="error-message">{error}</div>
    {:else if success}
        <div class="success-message">
            <h2>Автор успешно обновлен!</h2>
            <div class="action-buttons">
                <button class="action-button" on:click={resetForm}>
                    Продолжить редактирование
                </button>
                <a href={`/author/${id}`} use:link class="action-button">
                    Перейти к странице автора
                </a>
            </div>
        </div>
    {:else}
        <form on:submit|preventDefault={handleSubmit} class="author-form">
            <div class="form-group cover-upload">
                <label for="cover">Аватар автора</label>
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
                            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10s10-4.48 10-10S17.52 2 12 2zM7.07 18.28c.43-.9 3.05-1.78 4.93-1.78s4.5.88 4.93 1.78A7.893 7.893 0 0 1 12 20c-1.86 0-3.57-.64-4.93-1.72zm11.29-1.45c-1.43-1.74-4.9-2.33-6.36-2.33s-4.93.59-6.36 2.33A7.95 7.95 0 0 1 4 12c0-4.41 3.59-8 8-8s8 3.59 8 8c0 1.82-.62 3.49-1.64 4.83zM12 6c-1.94 0-3.5 1.56-3.5 3.5S10.06 13 12 13s3.5-1.56 3.5-3.5S13.94 6 12 6zm0 5c-.83 0-1.5-.67-1.5-1.5S11.17 8 12 8s1.5.67 1.5 1.5S12.83 11 12 11z"/></svg>
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
            </div>
            
            <div class="form-group">
                <label for="name">Имя</label>
                <input 
                    type="text" 
                    id="name" 
                    bind:value={name} 
                    placeholder="Введите имя автора" 
                    required
                />
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
    
    .author-form {
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
    
    input[type="text"] {
        padding: 0.75rem;
        border: 1px solid var(--border-color);
        border-radius: 0.5rem;
        background-color: var(--light-bg);
        color: var(--text-light);
    }
    
    .cover-upload {
        margin-bottom: 1rem;
    }
    
    .upload-container {
        position: relative;
        width: 200px;
        height: 200px;
        border: 2px dashed var(--border-color);
        border-radius: 50%;
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