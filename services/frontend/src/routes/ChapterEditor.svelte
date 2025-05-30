<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { Editor } from '@tiptap/core';
    import StarterKit from '@tiptap/starter-kit';
    import Placeholder from '@tiptap/extension-placeholder';
    import { createChapter, updateChapter, fetchChapter, fetchBookChapters } from '../api';
    import { link } from 'svelte-routing';
    
    export let bookId: string;
    export let index: string | null = null;
    
    let editor: Editor;
    let editorElement: HTMLElement;
    let chapters: any[] = [];
    let chapterName = '';
    let chapterIndexNumber = 1;
    let isLoading = false;
    let error: string | null = null;
    let isEditMode = index !== null;
    
    onMount(async () => {
        editor = new Editor({
            element: editorElement,
            extensions: [
                StarterKit,
                Placeholder.configure({
                    placeholder: 'Начните писать главу...'
                })
            ],
            content: {
                type: 'doc',
                content: [
                    {
                        type: 'paragraph',
                        content: []
                    }
                ]
            },
            onTransaction: () => {
                editor = editor;
            }
        });
        
        await loadChapters();
        
        if (isEditMode && index) {
            await loadChapter(parseInt(index));
        } else {
            chapterIndexNumber = chapters.length > 0 ? Math.max(...chapters.map(c => c.index)) + 1 : 1;
        }
    });
    
    onDestroy(() => {
        if (editor) {
            editor.destroy();
        }
    });
    
    async function loadChapters() {
        try {
            chapters = await fetchBookChapters(bookId);
        } catch (err) {
            console.error('Failed to load chapters:', err);
        }
    }
    
    async function loadChapter(index: number) {
        try {
            isLoading = true;
            const chapter = await fetchChapter(bookId, index);
            chapterName = chapter.name;
            chapterIndexNumber = chapter.index;
            
            if (editor) {
                editor.commands.setContent(chapter.content);
            }
        } catch (err) {
            error = 'Не удалось загрузить главу';
            console.error(err);
        } finally {
            isLoading = false;
        }
    }
    
    async function saveChapter() {
        if (!chapterName.trim()) {
            error = 'Название главы обязательно';
            return;
        }
        
        try {
            isLoading = true;
            error = null;
            
            const content = editor.getJSON();
            
            if (isEditMode && index) {
                await updateChapter(bookId, parseInt(index), {
                    name: chapterName,
                    content,
                    index: chapterIndexNumber
                });
            } else {
                await createChapter(bookId, {
                    name: chapterName,
                    content,
                    index: chapterIndexNumber
                });
            }
            
            window.location.href = `/book/${bookId}/chapters`;
            
        } catch (err) {
            error = `Не удалось ${isEditMode ? 'обновить' : 'создать'} главу`;
            console.error(err);
        } finally {
            isLoading = false;
        }
    }
    
    // Функции форматирования
    function toggleBold() {
        editor.chain().focus().toggleBold().run();
    }
    
    function toggleItalic() {
        editor.chain().focus().toggleItalic().run();
    }
    
    function setHeading(level: number) {
        editor.chain().focus().toggleHeading({ level: level as any }).run();
    }
    
    function setParagraph() {
        editor.chain().focus().setParagraph().run();
    }
</script>

<div class="editor-container">
    <div class="editor-header">
        <h1>{isEditMode ? 'Редактировать главу' : 'Создать главу'}</h1>
        
        {#if error}
            <div class="error">{error}</div>
        {/if}
    </div>
    
    <div class="chapter-info">
        <div class="form-group">
            <label for="chapter-name">Название главы</label>
            <input 
                type="text" 
                id="chapter-name"
                bind:value={chapterName}
                placeholder="Введите название главы"
                disabled={isLoading}
            />
        </div>
        
        <div class="form-group">
            <label for="chapter-index">Номер главы</label>
            <input 
                type="number" 
                id="chapter-index"
                bind:value={chapterIndexNumber}
                min="1"
                disabled={isLoading}
            />
        </div>
    </div>
    
    <div class="editor-wrapper">
        <div class="toolbar">
            <button 
                type="button" 
                class:active={editor?.isActive('bold')}
                on:click={toggleBold}
                disabled={isLoading}
                title="Жирный"
            >
                <strong>B</strong>
            </button>
            
            <button 
                type="button" 
                class:active={editor?.isActive('italic')}
                on:click={toggleItalic}
                disabled={isLoading}
                title="Курсив"
            >
                <em>I</em>
            </button>
            
            <div class="separator"></div>
            
            <button 
                type="button" 
                class:active={editor?.isActive('paragraph')}
                on:click={setParagraph}
                disabled={isLoading}
                title="Параграф"
            >
                P
            </button>
            
            <button 
                type="button" 
                class:active={editor?.isActive('heading', { level: 2 })}
                on:click={() => setHeading(2)}
                disabled={isLoading}
                title="Заголовок 2"
            >
                H2
            </button>
            
            <button 
                type="button" 
                class:active={editor?.isActive('heading', { level: 3 })}
                on:click={() => setHeading(3)}
                disabled={isLoading}
                title="Заголовок 3"
            >
                H3
            </button>
        </div>
        
        <div 
            class="editor-content" 
            bind:this={editorElement}
        ></div>
    </div>
    
    <div class="editor-actions">
        <button 
            class="save-btn"
            on:click={saveChapter}
            disabled={isLoading}
        >
            {isLoading ? 'Сохранение...' : (isEditMode ? 'Обновить' : 'Создать')}
        </button>
        
        <a href="/book/{bookId}/chapters" use:link class="cancel-btn">
            Отмена
        </a>
    </div>
</div>

<style>
    .editor-container {
        max-width: 1000px;
        margin: 0 auto;
        padding: 2rem;
    }
    
    .error {
        background: #fee;
        color: #c33;
        padding: 1rem;
        border-radius: 4px;
        margin-bottom: 1rem;
    }
    
    .chapter-info {
        display: grid;
        grid-template-columns: 2fr 1fr;
        gap: 1rem;
        margin-bottom: 2rem;
    }
    
    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 600;
    }
    
    .form-group input {
        width: 100%;
        padding: 0.75rem;
        border: 1px solid var(--border-color);
        border-radius: 4px;
        background: var(--light-bg);
        color: var(--text-light);
    }
    
    .editor-wrapper {
        border: 1px solid var(--border-color);
        border-radius: 8px;
        overflow: hidden;
        margin-bottom: 2rem;
    }
    
    .toolbar {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.75rem;
        background: var(--light-bg);
        border-bottom: 1px solid var(--border-color);
    }
    
    .toolbar button {
        padding: 0.5rem 0.75rem;
        border: 1px solid var(--border-color);
        background: white;
        border-radius: 4px;
        cursor: pointer;
        transition: all 0.2s;
    }
    
    .toolbar button:hover:not(:disabled) {
        background: var(--primary-color);
        color: white;
    }
    
    .toolbar button.active {
        background: var(--primary-color);
        color: white;
    }
    
    .toolbar button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    
    .separator {
        width: 1px;
        height: 1.5rem;
        background: var(--border-color);
        margin: 0 0.5rem;
    }
    
    :global(.editor-content) {
        min-height: 500px;
        padding: 1.5rem;
        font-size: 1.1rem;
        line-height: 1.8;
    }
    
    :global(.editor-content p) {
        margin-bottom: 1rem;
    }
    
    :global(.editor-content h2) {
        font-size: 1.5rem;
        margin: 2rem 0 1rem 0;
    }
    
    :global(.editor-content h3) {
        font-size: 1.25rem;
        margin: 1.5rem 0 0.75rem 0;
    }
    
    :global(.editor-content .is-editor-empty:first-child::before) {
        content: attr(data-placeholder);
        float: left;
        color: #adb5bd;
        pointer-events: none;
        height: 0;
    }
    
    .editor-actions {
        display: flex;
        gap: 1rem;
        align-items: center;
    }
    
    .save-btn {
        background: var(--primary-color);
        color: white;
        border: none;
        padding: 0.75rem 2rem;
        border-radius: 4px;
        cursor: pointer;
        font-size: 1rem;
        transition: background 0.2s;
    }
    
    .save-btn:hover:not(:disabled) {
        background: var(--secondary-color);
    }
    
    .save-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
    
    .cancel-btn {
        padding: 0.75rem 1.5rem;
        background: var(--light-bg);
        color: var(--text-light);
        text-decoration: none;
        border-radius: 4px;
        transition: background 0.2s;
    }
    
    .cancel-btn:hover {
        background: var(--border-color);
    }
</style>