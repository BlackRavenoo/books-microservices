import type { ChapterDraft, SaveStatus } from "./types";

export class DraftManager {
    private bookId: string;
    private chapterIndex: number;
    private autoSaveInterval: number | null = null;
    private onStatusChange: (status: SaveStatus) => void;
    private onDraftLoad: (draft: ChapterDraft) => void;
    private hasUnsavedChanges = false;
    private status: SaveStatus = 'saved';
    private lastSavedTime: Date | null = null;

    constructor(
        bookId: string, 
        chapterIndex: number,
        onStatusChange: (status: SaveStatus) => void,
        onDraftLoad: (draft: ChapterDraft) => void
    ) {
        this.bookId = bookId;
        this.chapterIndex = chapterIndex;
        this.onStatusChange = onStatusChange;
        this.onDraftLoad = onDraftLoad;
        
        this.setupAutoSave();
        this.setupBeforeUnload();
    }

    private getKey(): string {
        return `draft_${this.bookId}_${this.chapterIndex}`;
    }

    private updateStatus(status: SaveStatus): void {
        this.status = status;
        this.onStatusChange(status);
    }

    private setupAutoSave(): void {
        this.autoSaveInterval = setInterval(() => {
            if (this.hasUnsavedChanges) {
                this.saveDraft();
            }
        }, 30000);
    }

    private setupBeforeUnload(): void {
        window.addEventListener('beforeunload', this.handleBeforeUnload);
    }

    private handleBeforeUnload = (e: BeforeUnloadEvent): void => {
        if (this.hasUnsavedChanges) {
            e.preventDefault();
            this.saveDraft();
        }
    }

    public markAsUnsaved(): void {
        if (!this.hasUnsavedChanges) {
            this.hasUnsavedChanges = true;
            this.updateStatus('unsaved');
        }
    }

    public async loadDraft(): Promise<boolean> {
        const draftKey = this.getKey();
        const savedDraft = localStorage.getItem(draftKey);
        
        if (savedDraft) {
            try {
                const draft: ChapterDraft = JSON.parse(savedDraft);
                this.lastSavedTime = new Date(draft.last_saved);
                this.hasUnsavedChanges = false;
                this.updateStatus('saved');
                this.onDraftLoad(draft);
                return true;
            } catch (err) {
                console.error('Failed to load draft:', err);
                this.updateStatus('error');
                return false;
            }
        }
        return false;
    }

    public saveDraft(name?: string, content?: any): void {
        if (!this.hasUnsavedChanges || !name || !content) return;
        
        try {
            this.updateStatus('saving');
            
            const draftKey = this.getKey();
            const existingDraft = localStorage.getItem(draftKey);
            let currentDraft: Partial<ChapterDraft> = {};
            
            if (existingDraft) {
                currentDraft = JSON.parse(existingDraft);
            }
            
            const draft: ChapterDraft = {
                book_id: this.bookId,
                index: this.chapterIndex,
                name: name ?? currentDraft.name ?? '',
                content: content ?? currentDraft.content ?? {},
                last_saved: new Date().toISOString()
            };
            
            localStorage.setItem(draftKey, JSON.stringify(draft));
            
            this.hasUnsavedChanges = false;
            this.lastSavedTime = new Date();
            this.updateStatus('saved');
            
        } catch (err) {
            console.error('Failed to save draft:', err);
            this.updateStatus('error');
        }
    }

    public clearDraft(): void {
        const draftKey = this.getKey();
        localStorage.removeItem(draftKey);
        this.hasUnsavedChanges = false;
        this.updateStatus('saved');
    }

    public getLastSavedText(): string {
        if (!this.lastSavedTime) return '';
        
        const now = new Date();
        const diffMs = now.getTime() - this.lastSavedTime.getTime();
        const diffMinutes = Math.floor(diffMs / 60000);
        
        if (diffMinutes === 0) {
            return 'только что';
        } else if (diffMinutes === 1) {
            return '1 минуту назад';
        } else if (diffMinutes < 5) {
            return `${diffMinutes} минуты назад`;
        } else {
            return `${diffMinutes} минут назад`;
        }
    }

    public getStatus(): SaveStatus {
        return this.status;
    }

    public getLastSavedTime(): Date | null {
        return this.lastSavedTime;
    }

    public destroy(): void {
        if (this.autoSaveInterval) {
            clearInterval(this.autoSaveInterval);
            this.autoSaveInterval = null;
        }
        window.removeEventListener('beforeunload', this.handleBeforeUnload);
    }
}