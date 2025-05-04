import type { BookPreview, Book, BookSearchResult } from './types.ts';

const API_BASE_URL = '/api';

export async function fetchPopularBooks(): Promise<BookPreview[]> {
    try {
        const response = await fetch(`${API_BASE_URL}/books`);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error fetching popular books:', error);
        return [];
    }
}

export async function fetchBookDetails(id: string): Promise<Book | null> {
    try {
        const response = await fetch(`${API_BASE_URL}/books/${id}`);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error(`Error fetching book details for ID ${id}:`, error);
        return null;
    }
}

export async function searchBooks(query: string): Promise<BookSearchResult[]> {
    try {
        const response = await fetch(`${API_BASE_URL}/search?q=${encodeURIComponent(query)}`);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error searching books:', error);
        return [];
    }
}