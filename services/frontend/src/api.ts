import type { BookPreview, Book, BookSearchResult, Constants, CreateBookFields, UpdateBookFields, AuthorWithCover, BooksListPage } from './types.ts';

const API_BASE_URL = '/api';

export async function fetchBooks(params: {
    target?: string,
    target_id?: number,
    page?: number,
    page_size?: number,
    order_by?: string
} = {}): Promise<BooksListPage> {
    const queryParams = new URLSearchParams();
    if (params.page) queryParams.append('page', params.page.toString());
    if (params.page_size) queryParams.append('page_size', params.page_size.toString());
    if (params.order_by) queryParams.append('order_by', params.order_by);
    if (params.target && params.target_id) {
        queryParams.append('target', params.target);
        queryParams.append('target_id', params.target_id.toString());
    }
    
    const response = await fetch(`${API_BASE_URL}/books?${queryParams.toString()}`);
    
    if (!response.ok) {
        throw new Error(`Failed to fetch books: ${response.status} ${response.statusText}`);
    }
    
    return await response.json();
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

export async function updateBook(id: number, coverFile: File | null, fields: UpdateBookFields): Promise<any> {
    try {
        const formData = new FormData();
        
        if (coverFile) {
            formData.append('cover', coverFile);
        }

        const fieldsBlob = new Blob([JSON.stringify(fields)], {
            type: 'application/json'
        });

        formData.append('fields', fieldsBlob);
        
        const response = await fetch(`${API_BASE_URL}/books/${id}`, {
            method: 'PUT',
            body: formData,
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
        
        return await response.text();
    } catch (error) {
        console.error(`Error updating book ${id}:`, error);
        throw error;
    }
}

export async function createAuthor(coverFile: File | null, fields: { name: string }): Promise<any> {
    try {
        const formData = new FormData();
        
        if (coverFile) {
            formData.append('cover', coverFile);
        }

        const fieldsBlob = new Blob([JSON.stringify(fields)], {
            type: 'application/json'
        });

        formData.append('fields', fieldsBlob);
        
        const response = await fetch(`${API_BASE_URL}/authors`, {
            method: 'POST',
            body: formData,
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
        
        return await response.text();
    } catch (error) {
        console.error('Error creating author:', error);
        throw error;
    }
}

export async function updateAuthor(id: number, coverFile: File | null, fields: { name?: string }): Promise<any> {
    try {
        const formData = new FormData();
        
        if (coverFile) {
            formData.append('cover', coverFile);
        }

        const fieldsBlob = new Blob([JSON.stringify(fields)], {
            type: 'application/json'
        });

        formData.append('fields', fieldsBlob);
        
        const response = await fetch(`${API_BASE_URL}/authors/${id}`, {
            method: 'PUT',
            body: formData,
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
        
        return await response.text();
    } catch (error) {
        console.error(`Error updating author ${id}:`, error);
        throw error;
    }
}

export async function fetchAuthorDetails(id: string): Promise<AuthorWithCover | null> {
    try {
        const response = await fetch(`${API_BASE_URL}/authors/${id}`);
        if (!response.ok) {
            throw new Error(`Failed to fetch author: ${response.statusText}`);
        }
        return await response.json();
    } catch (error) {
        console.error(`Error fetching author details for ID ${id}:`, error);
        return null;
    }
}