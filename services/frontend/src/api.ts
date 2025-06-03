import type { Book, Constants, CreateBookFields, UpdateBookFields, AuthorWithCover, BooksListPage, ChapterFullSchema, ChapterSchema } from './types.ts';
import { authStore } from './store/authStore';
import { refreshAccessToken } from './utils/auth';
import { get } from 'svelte/store';

let isRefreshing = false;
let refreshPromise: Promise<boolean> | null = null;

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
        const response = await fetchWithAuth(`${API_BASE_URL}/books/${id}`);
        
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
        
        const response = await fetchWithAuth(`${API_BASE_URL}/books`, {
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
        
        const response = await fetchWithAuth(`${API_BASE_URL}/books/${id}`, {
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

        const response = await fetchWithAuth(`${API_BASE_URL}/authors`, {
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
        
        const response = await fetchWithAuth(`${API_BASE_URL}/authors/${id}`, {
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

export async function createChapter(bookId: string, fields: {
    name: string;
    content: any;
    index: number;
}): Promise<any> {
    try {
        const response = await fetchWithAuth(`${API_BASE_URL}/books/${bookId}/chapter`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(fields),
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
        
        return await response.text();
    } catch (error) {
        console.error('Error creating chapter:', error);
        throw error;
    }
}

export async function fetchBookChapters(bookId: string): Promise<ChapterSchema[]> {
    try {
        const response = await fetch(`${API_BASE_URL}/books/${bookId}/chapters`);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error(`Error fetching chapters for book ${bookId}:`, error);
        return [];
    }
}

export async function fetchChapter(bookId: string, chapterIndex: number): Promise<ChapterFullSchema> {
    try {
        const response = await fetch(`${API_BASE_URL}/books/${bookId}/chapter?number=${chapterIndex}`);
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error(`Error fetching chapter ${chapterIndex}:`, error);
        throw error;
    }
}

export async function updateChapter(bookId: string, chapterIndex: number, fields: {
    name?: string;
    content?: any;
    index?: number;
}): Promise<any> {
    try {
        const response = await fetchWithAuth(`${API_BASE_URL}/books/${bookId}/chapter?number=${chapterIndex}`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(fields),
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
        
        return await response.json();
    } catch (error) {
        console.error('Error updating chapter:', error);
        throw error;
    }
}

export async function deleteChapter(bookId: string, chapterIndex: number): Promise<any> {
    try {
        const response = await fetchWithAuth(`${API_BASE_URL}/books/${bookId}/chapter?number=${chapterIndex}`, {
            method: 'DELETE'
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
        
        return await response.json();
    } catch (error) {
        console.error('Error deleting chapter:', error);
        throw error;
    }
}

export async function rateBook(bookId: number, score: number): Promise<void> {
    try {
        const response = await fetchWithAuth(`${API_BASE_URL}/ratings/rate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                item_id: bookId,
                score: score
            }),
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            throw new Error(errorText || `HTTP error! Status: ${response.status}`);
        }
    } catch (error) {
        console.error('Error rating book:', error);
        throw error;
    }
}

export async function removeBookRating(bookId: number): Promise<void> {
    return rateBook(bookId, 0);
}

async function fetchWithAuth(url: string, options: RequestInit = {}): Promise<Response> {
    const makeRequest = async (token: string | null): Promise<Response> => {
        const headers: Record<string, string> = {
            ...(options.headers as Record<string, string> || {}),
          };
  
        if (token) {
            headers['Authorization'] = `Bearer ${token}`;
        }
  
        return fetch(url, {
            ...options,
            headers,
        });
    };

    const authState = get(authStore);
    let response = await makeRequest(authState.token?.access_token || null);

    if (response.status === 401 && authState.token?.refresh_token) {
        const tokenRefreshed = await refreshTokenIfNeeded();
        
        if (tokenRefreshed) {
            const newAuthState = get(authStore);
            response = await makeRequest(newAuthState.token?.access_token || null);
        }
    }

    return response;
}

async function refreshTokenIfNeeded(): Promise<boolean> {
    if (isRefreshing && refreshPromise) {
        return await refreshPromise;
    }
  
    isRefreshing = true;
    refreshPromise = performTokenRefresh();
  
    try {
        const result = await refreshPromise;
        return result;
    } finally {
        isRefreshing = false;
        refreshPromise = null;
    }
}

async function performTokenRefresh(): Promise<boolean> {
    try {
        const authState = get(authStore);
        const refreshToken = authState.token?.refresh_token;
  
        if (!refreshToken) {
            authStore.logout();
            return false;
        }
    
        const newToken = await refreshAccessToken(refreshToken);
        
        authStore.setTokens(newToken.access_token, newToken.refresh_token, newToken.token_type);
        
        return true;
    } catch (error) {
        console.error('Error refreshing token:', error);
        authStore.logout();
        return false;
    }
}