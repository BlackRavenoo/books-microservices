import { writable } from 'svelte/store';
import type { Book, ChapterSchema } from '../types';

export interface BookState {
    currentBook: Book | null;
    chapters: ChapterSchema[];
    isLoaded: boolean;
}

function createBookStore() {
    const { subscribe, set, update } = writable<BookState>({
        currentBook: null,
        chapters: [],
        isLoaded: false
    });

    return {
        subscribe,
        setBookData: (book: Book) => {
            update(state => ({
                ...state,
                currentBook: book,
                isLoaded: true
            }));
        },
        setChapters: (chapters: ChapterSchema[]) => {
            update(state => ({
                ...state,
                chapters
            }));
        },
        clear: () => set({ currentBook: null, chapters: [], isLoaded: false })
    };
}

export const bookStore = createBookStore();