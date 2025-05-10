import type { BookPreview, Book, BookSearchResult, Constants, CreateBookFields } from './types.ts';

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

export async function search(query: string, entity: string): Promise<any[]> {
    try {
        const response = await fetch(`${API_BASE_URL}/search/${entity}?q=${encodeURIComponent(query)}`);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error(`Error searching ${entity}:`, error);
        return [];
    }
}

export async function fetchConstants(): Promise<Constants> {
    try {
        const response = await fetch(`${API_BASE_URL}/constants`);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error fetching constants:', error);
        throw error;
    }
}

export async function createBook(coverFile: File, fields: CreateBookFields): Promise<any> {
    try {
        const formData = new FormData();
        formData.append('cover', coverFile);

        const fieldsBlob = new Blob([JSON.stringify(fields)], {
            type: 'application/json'
        });

        formData.append('fields', fieldsBlob);
        
        const response = await fetch(`${API_BASE_URL}/books`, {
            method: 'POST',
            body: formData,
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
        
        return await response.text();
    } catch (error) {
        console.error('Error creating book:', error);
        throw error;
    }
}