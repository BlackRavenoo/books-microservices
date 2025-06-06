<script lang="ts">
    import { link, navigate } from 'svelte-routing';
    import { writable } from 'svelte/store';
    import { onMount } from 'svelte';
    
    export let bookId: string;
    export let bookTitle: string = '';
    export let currentChapter: any = null;
    export let prevChapter: any = null;
    export let nextChapter: any = null;
    
    const readerSettings = writable({
        backgroundColor: '#111827',
        textColor: '#f3f4f6',
        fontSize: 18,
        lineHeight: 1.8,
        fontFamily: 'Inter'
    });
    
    let showSettings = false;
    let settings = {
        backgroundColor: '#111827',
        textColor: '#f3f4f6',
        fontSize: 18,
        lineHeight: 1.8,
        fontFamily: 'Inter'
    };
    
    const fontOptions = [
        { value: 'Inter', label: 'Inter' },
        { value: 'Georgia, serif', label: 'Georgia' },
        { value: 'Times New Roman, serif', label: 'Times' },
        { value: 'Arial, sans-serif', label: 'Arial' }
    ];
    
    const backgroundPresets = [
        { name: 'Темный', bg: '#111827', text: '#f3f4f6' },
        { name: 'Светлый', bg: '#ffffff', text: '#1f2937' },
        { name: 'Сепия', bg: '#f7f3e9', text: '#5d4037' },
        { name: 'Черный', bg: '#000000', text: '#e0e0e0' }
    ];
    
    onMount(() => {
        const saved = localStorage.getItem('readerSettings');
        if (saved) {
            settings = { ...settings, ...JSON.parse(saved) };
            readerSettings.set(settings);
            applySettings();
        }
    });
    
    function toggleSettings() {
        showSettings = !showSettings;
    }
    
    function closeSettings() {
        showSettings = false;
    }
    
    function applySettings() {
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
        
        localStorage.setItem('readerSettings', JSON.stringify(settings));
        readerSettings.set(settings);
    }

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
    
    function selectPreset(preset: typeof backgroundPresets[0]) {
        settings.backgroundColor = preset.bg;
        settings.textColor = preset.text;
        applySettings();
    }
    
    $: chapterInfo = currentChapter ? `Глава ${currentChapter.index}` : '';

    function goToPrevChapter() {
        if (prevChapter) {
            navigate(`/book/${bookId}/chapter?number=${prevChapter.index}`);
        }
    }
    
    function goToNextChapter() {
        if (nextChapter) {
            navigate(`/book/${bookId}/chapter?number=${nextChapter.index}`);
        }
    }
</script>

<header class="reader-header">
    <div class="header-left">
        <a href="/" use:link class="back-home">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
            </svg>
        </a>
        
        {#if bookTitle}
            <a href="/book/{bookId}" use:link class="book-title">
                {bookTitle}
            </a>
        {/if}
    </div>
    
    <div class="header-center">
        <button 
            class="nav-chapter" 
            disabled={!prevChapter}
            on:click={goToPrevChapter}
        >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12z"/>
            </svg>
        </button>
        
        <span class="chapter-info">{chapterInfo}</span>
        
        <button 
            class="nav-chapter" 
            disabled={!nextChapter}
            on:click={goToNextChapter}
        >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6z"/>
            </svg>
        </button>
    </div>
    
    <div class="header-right">
        <button class="settings-btn" on:click={toggleSettings}>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M19.14,12.94c0.04-0.3,0.06-0.61,0.06-0.94c0-0.32-0.02-0.64-0.07-0.94l2.03-1.58c0.18-0.14,0.23-0.41,0.12-0.61 l-1.92-3.32c-0.12-0.22-0.37-0.29-0.59-0.22l-2.39,0.96c-0.5-0.38-1.03-0.7-1.62-0.94L14.4,2.81c-0.04-0.24-0.24-0.41-0.48-0.41 h-3.84c-0.24,0-0.43,0.17-0.47,0.41L9.25,5.35C8.66,5.59,8.12,5.92,7.63,6.29L5.24,5.33c-0.22-0.08-0.47,0-0.59,0.22L2.74,8.87 C2.62,9.08,2.66,9.34,2.86,9.48l2.03,1.58C4.84,11.36,4.82,11.69,4.82,12s0.02,0.64,0.07,0.94l-2.03,1.58 c-0.18,0.14-0.23,0.41-0.12,0.61l1.92,3.32c0.12,0.22,0.37,0.29,0.59,0.22l2.39-0.96c0.5,0.38,1.03,0.7,1.62,0.94l0.36,2.54 c0.05,0.24,0.24,0.41,0.48,0.41h3.84c0.24,0,0.44-0.17,0.47-0.41l0.36-2.54c0.59-0.24,1.13-0.56,1.62-0.94l2.39,0.96 c0.22,0.08,0.47,0,0.59-0.22l1.92-3.32c0.12-0.22,0.07-0.47-0.12-0.61L19.14,12.94z M12,15.6c-1.98,0-3.6-1.62-3.6-3.6 s1.62-3.6,3.6-3.6s3.6,1.62,3.6,3.6S13.98,15.6,12,15.6z"/>
            </svg>
        </button>
    </div>
</header>

{#if showSettings}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="settings-overlay" on:click={closeSettings}>
        <div class="settings-panel" on:click|stopPropagation>
            <div class="settings-header">
                <h3>Настройки чтения</h3>
                <button class="close-btn" on:click={closeSettings}>×</button>
            </div>
            
            <div class="settings-content">
                <div class="setting-group">
                    <span class="setting-label">Тема оформления</span>
                    <div class="preset-buttons">
                        {#each backgroundPresets as preset}
                            <button 
                                class="preset-btn"
                                style="background: {preset.bg}; color: {preset.text};"
                                on:click={() => selectPreset(preset)}
                            >
                                {preset.name}
                            </button>
                        {/each}
                    </div>
                </div>
                
                <div class="setting-group">
                    <label for="bg-color">Цвет фона</label>
                    <input 
                        id="bg-color"
                        type="color" 
                        bind:value={settings.backgroundColor}
                        on:change={applySettings}
                    />
                </div>
                
                <div class="setting-group">
                    <label for="text-color">Цвет текста</label>
                    <input 
                        id="text-color"
                        type="color" 
                        bind:value={settings.textColor}
                        on:change={applySettings}
                    />
                </div>
                
                <div class="setting-group">
                    <label for="font-size">Размер шрифта: {settings.fontSize}px</label>
                    <input 
                        id="font-size"
                        type="range" 
                        min="14" 
                        max="24" 
                        bind:value={settings.fontSize}
                        on:change={applySettings}
                    />
                </div>
                
                <div class="setting-group">
                    <label for="line-height">Межстрочный интервал: {settings.lineHeight}</label>
                    <input 
                        id="line-height"
                        type="range" 
                        min="1.2" 
                        max="2.4" 
                        step="0.1" 
                        bind:value={settings.lineHeight}
                        on:change={applySettings}
                    />
                </div>
                
                <div class="setting-group">
                    <label for="font-family">Шрифт</label>
                    <select 
                        id="font-family"
                        bind:value={settings.fontFamily}
                        on:change={applySettings}
                    >
                        {#each fontOptions as font}
                            <option value={font.value}>{font.label}</option>
                        {/each}
                    </select>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .reader-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 2rem;
        background: var(--reader-bg, var(--dark-bg));
        border-bottom: 1px solid var(--reader-border, var(--border-color));
        position: sticky;
        top: 0;
        z-index: 100;
    }
    
    .header-left {
        display: flex;
        align-items: center;
        gap: 1rem;
    }
    
    .back-home {
        color: var(--reader-text, var(--text-light));
        text-decoration: none;
        padding: 0.5rem;
        border-radius: 4px;
        transition: background 0.2s;
    }
    
    .book-title {
        color: var(--reader-text, var(--text-light));
        text-decoration: none;
        font-weight: 600;
        font-size: 1.1rem;
        transition: color 0.2s;
    }
    
    .book-title:hover {
        color: var(--primary-color);
    }
    
    .header-center {
        display: flex;
        align-items: center;
        gap: 1rem;
    }
    
    .nav-chapter {
        background: none;
        border: none;
        color: var(--text-light);
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 4px;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
    }
    
    .nav-chapter:hover:not(:disabled) {
        background: var(--reader-bg-light, var(--light-bg));
        color: var(--primary-color);
    }
    
    .nav-chapter:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }
    
    .chapter-info {
        color: var(--reader-text-muted, var(--text-muted));
        font-size: 0.9rem;
        min-width: 80px;
        text-align: center;
    }
    
    .settings-btn {
        background: none;
        border: none;
        color: var(--reader-text, var(--text-light));
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 4px;
        transition: all 0.2s;
    }
    
    .settings-btn:hover {
        background: var(--reader-bg-light, var(--light-bg));
        color: var(--primary-color);
    }
    
    .settings-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }
    
    .settings-panel {
        background: var(--reader-bg, var(--dark-bg));
        border-radius: 8px;
        width: 90%;
        max-width: 500px;
        max-height: 80vh;
        overflow-y: auto;
        border: 1px solid var(--reader-border, var(--border-color));
    }
    
    .settings-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem 1.5rem;
        border-bottom: 1px solid var(--reader-border, var(--border-color));
    }
    
    .settings-header h3 {
        margin: 0;
        color: var(--reader-text, var(--text-light));
    }
    
    .close-btn {
        background: none;
        border: none;
        color: var(--text-light);
        font-size: 1.5rem;
        cursor: pointer;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        transition: background 0.2s;
    }
    
    .close-btn:hover {
        background: var(--reader-bg-light, var(--light-bg));
    }
    
    .settings-content {
        padding: 1.5rem;
    }
    
    .setting-group {
        margin-bottom: 1.5rem;
    }
    
    .setting-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
    }
    
    .setting-group input, .setting-group select {
        width: 100%;
        padding: 0.5rem;
        background: var(--reader-bg-light, var(--light-bg));
        border: 1px solid var(--reader-border, var(--border-color));
        border-radius: 4px;
        color: var(--reader-text, var(--text-light));
        font-size: 1rem;
    }
    
    .setting-group input[type="color"] {
        width: 60px;
        height: 40px;
        padding: 0;
        border: none;
        cursor: pointer;
    }
    
    .preset-buttons {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.5rem;
    }
    
    .preset-btn {
        padding: 0.75rem;
        border: 2px solid transparent;
        border-radius: 4px;
        cursor: pointer;
        font-size: 0.9rem;
        font-weight: 500;
        transition: border-color 0.2s;
    }
    
    .preset-btn:hover {
        border-color: var(--primary-color);
    }
    
    @media (max-width: 768px) {
        .reader-header {
            padding: 1rem;
        }
        
        .book-title {
            font-size: 1rem;
            max-width: 150px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
        
        .chapter-info {
            font-size: 0.8rem;
            min-width: 60px;
        }
        
        .settings-panel {
            width: 95%;
            margin: 1rem;
        }
        
        .preset-buttons {
            grid-template-columns: 1fr;
        }
    }
</style>