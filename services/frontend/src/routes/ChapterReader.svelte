<script lang="ts">
    import { onMount } from 'svelte';
    import { fetchChapter, fetchBookChapters } from '../api';
    import { link } from 'svelte-routing';
    
    export let bookId: string;
    
    let chapter: any = null;
    let allChapters: any[] = [];
    let loading = true;
    let error = false;
    
    let currentIndex = 0;
    let nextChapter: any = null;
    let prevChapter: any = null;
    
    onMount(async () => {
        const urlParams = new URLSearchParams(window.location.search);
        const numberParam = urlParams.get('number');
        
        if (!numberParam) {
            error = true;
            loading = false;
            console.error('Chapter number not provided');
            return;
        }
        
        const chapterNumber = parseInt(numberParam);
        
        if (isNaN(chapterNumber)) {
            error = true;
            loading = false;
            console.error('Invalid chapter number');
            return;
        }
        
        await loadChapter(chapterNumber);
        await loadChaptersList();
    });
    
    async function loadChapter(chapterNumber: number) {
        try {
            chapter = await fetchChapter(bookId, chapterNumber);
        } catch (err) {
            error = true;
            console.error(err);
        } finally {
            loading = false;
        }
    }
    
    async function loadChaptersList() {
        try {
            allChapters = await fetchBookChapters(bookId);
            if (chapter) {
                currentIndex = allChapters.findIndex(c => c.index === chapter.index);
                nextChapter = allChapters[currentIndex + 1] || null;
                prevChapter = allChapters[currentIndex - 1] || null;
            }
        } catch (err) {
            console.error('Failed to load chapters list:', err);
        }
    }
    
    function renderContent(content: any): string {
        if (!content || !content.content) return '';
        
        return content.content.map((node: any) => renderNode(node)).join('');
    }
    
    function renderNode(node: any): string {
        if (!node.type) return '';
        
        switch (node.type) {
            case 'paragraph':
                const pContent = node.content ? node.content.map(renderNode).join('') : '';
                return `<p>${pContent}</p>`;
            
            case 'heading':
                const level = node.attrs?.level || 2;
                const hContent = node.content ? node.content.map(renderNode).join('') : '';
                return `<h${level}>${hContent}</h${level}>`;
            
            case 'text':
                let text = node.text || '';
                
                if (node.marks) {
                    node.marks.forEach((mark: any) => {
                        switch (mark.type) {
                            case 'bold':
                                text = `<strong>${text}</strong>`;
                                break;
                            case 'italic':
                                text = `<em>${text}</em>`;
                                break;
                        }
                    });
                }
                
                return text;
            
            default:
                return '';
        }
    }
</script>

<div class="reader-container">
    {#if loading}
        <div class="loading">Загрузка главы...</div>
    {:else if error || !chapter}
        <div class="error">Глава не найдена</div>
    {:else}
        <div class="chapter-header">
            <h1>{chapter.name}</h1>
            <div class="chapter-meta">
                Глава {chapter.index} • {new Date(chapter.created_at).toLocaleDateString()}
            </div>
        </div>
        
        <div class="chapter-content">
            {@html renderContent(chapter.content)}
        </div>
        
        <div class="chapter-navigation">
            {#if prevChapter}
                <a href="/book/{bookId}/chapter?number={prevChapter.index}" use:link class="nav-btn prev">
                    ← {prevChapter.name}
                </a>
            {/if}
            
            <a href="/book/{bookId}" use:link class="nav-btn back">
                Вернуться к книге
            </a>
            
            {#if nextChapter}
                <a href="/book/{bookId}/chapter?number={nextChapter.index}" use:link class="nav-btn next">
                    {nextChapter.name} →
                </a>
            {/if}
        </div>
    {/if}
</div>

<style>
    .reader-container {
        max-width: 800px;
        margin: 0 auto;
        padding: 2rem;
        line-height: 1.8;
    }
    
    .loading {
        text-align: center;
        padding: 3rem;
        color: var(--text-muted);
    }
    
    .error {
        text-align: center;
        padding: 3rem;
        color: #c33;
        background: #fee;
        border-radius: 8px;
    }
    
    .chapter-header {
        margin-bottom: 2rem;
        text-align: center;
    }
    
    .chapter-header h1 {
        margin-bottom: 0.5rem;
        font-size: 2rem;
        color: var(--text-light);
    }
    
    .chapter-meta {
        color: var(--text-muted);
        font-size: 0.9rem;
    }
    
    .chapter-content {
        margin-bottom: 3rem;
        font-size: 1.1rem;
    }
    
    .chapter-content :global(h2) {
        margin-top: 2rem;
        margin-bottom: 1rem;
        font-size: 1.5rem;
        color: var(--text-light);
    }
    
    .chapter-content :global(h3) {
        margin-top: 1.5rem;
        margin-bottom: 0.75rem;
        font-size: 1.25rem;
        color: var(--text-light);
    }
    
    .chapter-content :global(p) {
        margin-bottom: 1rem;
        text-align: justify;
        color: var(--text-light);
    }
    
    .chapter-content :global(strong) {
        font-weight: 600;
    }
    
    .chapter-content :global(em) {
        font-style: italic;
    }
    
    .chapter-navigation {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 0;
        border-top: 1px solid var(--border-color);
        gap: 1rem;
    }
    
    .nav-btn {
        padding: 0.75rem 1.5rem;
        background: var(--primary-color);
        color: white;
        text-decoration: none;
        border-radius: 4px;
        transition: background 0.2s;
        text-align: center;
        flex: 0 1 auto;
        min-width: 120px;
    }
    
    .nav-btn:hover {
        background: var(--secondary-color);
    }
    
    .nav-btn.back {
        background: var(--light-bg);
        color: var(--text-light);
        border: 1px solid var(--border-color);
    }
    
    .nav-btn.back:hover {
        background: var(--border-color);
    }
    
    @media (max-width: 768px) {
        .chapter-navigation {
            flex-direction: column;
            gap: 0.5rem;
        }
        
        .nav-btn {
            width: 100%;
        }
    }
</style>