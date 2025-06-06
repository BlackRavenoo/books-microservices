<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { fetchChapter, fetchBookChapters, fetchBookDetails } from '../api';
    import { bookStore, type BookState } from '../store/bookStore';
    import { link } from 'svelte-routing';
    import ReaderHeader from '../components/ReaderHeader.svelte';
    
    export let bookId: string;

    let bookState: BookState;
    const unsubscribe = bookStore.subscribe(state => {
        bookState = state;
    });
    
    let chapter: any = null;
    let allChapters: any[] = [];
    let loading = true;
    let error = false;
    
    let currentIndex = 0;
    let nextChapter: any = null;
    let prevChapter: any = null;

    let bookTitle = '';
    let currentChapterNumber: number | null = null;
    let currentUrl = '';

    $: {
        if (typeof window !== 'undefined') {
            const newUrl = window.location.href;
            if (newUrl !== currentUrl) {
                currentUrl = newUrl;
                handleUrlChange();
            }
        }
    }


    function getChapterNumberFromUrl(): number | null {
        const urlParams = new URLSearchParams(window.location.search);
        const numberParam = urlParams.get('number');
        return numberParam ? parseInt(numberParam) : null;
    }

    async function handleUrlChange() {
        const urlChapterNumber = getChapterNumberFromUrl();
        if (urlChapterNumber !== currentChapterNumber && urlChapterNumber !== null) {
            currentChapterNumber = urlChapterNumber;
            await loadChapterData(urlChapterNumber);
        }
    }

    const handlePopState = () => {
        handleUrlChange();
    };

    const originalPushState = history.pushState;
    const originalReplaceState = history.replaceState;

    onMount(async () => {
        currentUrl = window.location.href;
        const chapterNumber = getChapterNumberFromUrl();

        if (!chapterNumber) {
            error = true;
            loading = false;
            console.error('Chapter number not provided');
            return;
        }
        
        currentChapterNumber = chapterNumber;

        const saved = localStorage.getItem('readerSettings');
        if (saved) {
            const settings = JSON.parse(saved);
            document.documentElement.style.setProperty('--reader-bg', settings.backgroundColor);
            document.documentElement.style.setProperty('--reader-text', settings.textColor);
            document.documentElement.style.setProperty('--reader-font-size', settings.fontSize + 'px');
            document.documentElement.style.setProperty('--reader-line-height', settings.lineHeight.toString());
            document.documentElement.style.setProperty('--reader-font-family', settings.fontFamily);
            
            document.documentElement.style.setProperty('--reader-bg-light', adjustBrightness(settings.backgroundColor, 0.1));
            document.documentElement.style.setProperty('--reader-text-muted', adjustOpacity(settings.textColor, 0.7));
            document.documentElement.style.setProperty('--reader-border', adjustOpacity(settings.textColor, 0.2));
            
            document.body.style.backgroundColor = settings.backgroundColor;
            document.body.style.color = settings.textColor;
        }
        
        await loadBookInfo();
        await loadChapter(chapterNumber);
        await loadChaptersList();
        
        window.addEventListener('popstate', handlePopState);
        
        history.pushState = function(...args) {
            originalPushState.apply(history, args);
            setTimeout(() => handleUrlChange(), 0);
        };
        
        history.replaceState = function(...args) {
            originalReplaceState.apply(history, args);
            setTimeout(() => handleUrlChange(), 0);
        };

        return () => {
            window.removeEventListener('popstate', handlePopState);
            history.pushState = originalPushState;
            history.replaceState = originalReplaceState;
        };
    });

    async function loadChapterData(chapterNumber: number) {
        if (isNaN(chapterNumber)) {
            error = true;
            loading = false;
            console.error('Invalid chapter number');
            return;
        }

        loading = true;
        error = false;
        
        try {
            chapter = await fetchChapter(bookId, chapterNumber);
            if (allChapters.length > 0) {
                updateNavigation();
            }
        } catch (err) {
            error = true;
            console.error(err);
        } finally {
            loading = false;
        }
    }
    
    function updateNavigation() {
        if (chapter && allChapters.length > 0) {
            currentIndex = allChapters.findIndex(c => c.index === chapter.index);
            nextChapter = allChapters[currentIndex + 1] || null;
            prevChapter = allChapters[currentIndex - 1] || null;
        }
    }

    onDestroy(() => {
        document.body.style.backgroundColor = '';
        document.body.style.color = '';
        history.pushState = originalPushState;
        history.replaceState = originalReplaceState;
        window.removeEventListener('popstate', handlePopState);
        unsubscribe();
    });

    function adjustBrightness(hex: string, factor: number): string {
        const r = parseInt(hex.slice(1, 3), 16);
        const g = parseInt(hex.slice(3, 5), 16);
        const b = parseInt(hex.slice(5, 7), 16);
        
        const newR = Math.min(255, Math.max(0, Math.floor(r + (255 - r) * factor)));
        const newG = Math.min(255, Math.max(0, Math.floor(g + (255 - g) * factor)));
        const newB = Math.min(255, Math.max(0, Math.floor(b + (255 - b) * factor)));
        
        return `#${newR.toString(16).padStart(2, '0')}${newG.toString(16).padStart(2, '0')}${newB.toString(16).padStart(2, '0')}`;
    }

    function adjustOpacity(hex: string, opacity: number): string {
        const r = parseInt(hex.slice(1, 3), 16);
        const g = parseInt(hex.slice(3, 5), 16);
        const b = parseInt(hex.slice(5, 7), 16);
        
        return `rgba(${r}, ${g}, ${b}, ${opacity})`;
    }
    
    async function loadBookInfo() {
        if (!bookState.isLoaded) {
            try {
                const book = await fetchBookDetails(bookId);
                if (book) {
                    bookStore.setBookData(book);
                }
            } catch (err) {
                console.error("Failed to load book info:", err)
            }
        }
        if (bookState.currentBook) {
            bookTitle = bookState.currentBook.title;
        }
    }

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
            if (bookState.chapters.length > 0 && String(bookState.currentBook?.id) === bookId) {
                allChapters = bookState.chapters;
            } else {
                allChapters = await fetchBookChapters(bookId);
                bookStore.setChapters(allChapters);
            }
            updateNavigation();
        } catch (err) {
            console.error('Failed to load chapters list:', err);
        }
    }
    
    function renderContent(content: any): string {
        if (!content) return '';
        
        return content.map((node: any) => renderNode(node)).join('');
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

<ReaderHeader 
    {bookId} 
    {bookTitle}
    currentChapter={chapter}
    {prevChapter}
    {nextChapter}
/>

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
        margin: 0 auto;
        padding: 2rem;
        line-height: var(--reader-line-height, 1.8);
        background: var(--reader-bg, var(--dark-bg));
        color: var(--reader-text, var(--text-light));
        font-size: var(--reader-font-size, 1.1rem);
        font-family: var(--reader-font-family, 'Inter');
        min-height: calc(100vh - 80px);
    }
    
    .loading {
        text-align: center;
        padding: 3rem;
        color: var(--reader-text-muted, var(--text-muted));
        font-family: var(--reader-font-family, 'Inter');
    }
    
    .error {
        text-align: center;
        padding: 3rem;
        color: var(--reader-text, #c33);
        background: var(--reader-bg-light, var(--light-bg));
        border-radius: 8px;
        border: 1px solid var(--reader-border, var(--border-color));
    }
    
    .chapter-header {
        margin-bottom: 2rem;
        text-align: center;
    }
    
    .chapter-header h1 {
        margin-bottom: 0.5rem;
        font-size: calc(var(--reader-font-size, 1.1rem) * 1.8);
        color: var(--reader-text, var(--text-light));
        font-family: var(--reader-font-family, 'Inter');
    }
    
    .chapter-meta {
        color: var(--reader-text-muted, var(--text-muted));
        font-size: calc(var(--reader-font-size, 1.1rem) * 0.8);
        font-family: var(--reader-font-family, 'Inter');
    }
    
    .chapter-content {
        margin-bottom: 3rem;
        font-size: var(--reader-font-size, 1.1rem);
        color: var(--reader-text, var(--text-light));
    }
    
    .chapter-content :global(p) {
        margin-bottom: 1rem;
        text-align: justify;
        color: var(--reader-text, var(--text-light));
    }

    .chapter-content :global(h2) {
        margin-top: 2rem;
        margin-bottom: 1rem;
        font-size: calc(var(--reader-font-size, 1.1rem) * 1.36);
        color: var(--reader-text, var(--text-light));
    }

    .chapter-content :global(h3) {
        margin-top: 1.5rem;
        margin-bottom: 0.75rem;
        font-size: calc(var(--reader-font-size, 1.1rem) * 1.14);
        color: var(--reader-text, var(--text-light));
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
        border-top: 1px solid var(--reader-border, var(--border-color));
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