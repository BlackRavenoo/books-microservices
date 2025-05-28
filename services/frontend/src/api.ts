import type { Book, Constants, CreateBookFields, UpdateBookFields, AuthorWithCover, BooksListPage } from './types.ts';
import { authStore } from './store/authStore';

const API_BASE_URL = '/api';

export async function fetchBooks(params: {
    target?: string,
    target_id?: number,
    page?: number,
    page_size?: number,
    order_by?: string,
    genres_include?: number[],
    genres_exclude?: number[],
    tags_include?: number[],
    tags_exclude?: number[],
    statuses?: number[],
} = {}): Promise<BooksListPage> {
    const queryParts: string[] = [];

    if (params.page) queryParts.push(`page=${params.page}`);
    if (params.page_size) queryParts.push(`page_size=${params.page_size}`);
    if (params.order_by) queryParts.push(`order_by=${encodeURIComponent(params.order_by)}`);
    if (params.target && params.target_id) {
        queryParts.push(`target=${params.target}&target_id=${params.target_id}`);
    }

    const addArrayParams = (key: string, values?: number[]) => {
        if (values?.length) {
            values.forEach(v => queryParts.push(`${key}[]=${v}`));
        }
    };

    addArrayParams('genres_include', params.genres_include);
    addArrayParams('genres_exclude', params.genres_exclude);
    addArrayParams('tags_include', params.tags_include);
    addArrayParams('tags_exclude', params.tags_exclude);
    addArrayParams('statuses', params.statuses);

    const queryString = queryParts.join('&');
    const url = `${API_BASE_URL}/books?${queryString}`;

    const response = await fetch(url);

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
            headers: {
                ...getAuthHeaders()
            },
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
            headers: {
                ...getAuthHeaders()
            },
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
            headers: {
                ...getAuthHeaders()
            },
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
            headers: {
                ...getAuthHeaders()
            },
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

function getAuthHeaders(): HeadersInit {
    const headers: HeadersInit = {};
    
    let token: string | null = null;
    authStore.subscribe(state => {
        token = state.token?.access_token || null;
    })();
    
    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }
    
    return headers;
}