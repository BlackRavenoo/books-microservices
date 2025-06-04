export interface BookPreview {
    id: number;
    title: string;
    thumbnail: string;
    avg_rating: number;
}

export interface BooksListPage {
    max_page: number,
    total_items: number,
    items: BookPreview[]
}
  
export interface BookSearchResult extends BookPreview {
    status: string;
}

export interface BookStatus {
    id: number,
    name: string
}

export interface Tag {
    id: number,
    name: string
}

export interface Genre {
    id: number,
    name: string
}

export interface Constants {
    tags: Tag[];
    genres: Genre[];
    status: BookStatus[];
}

export interface Author {
    id: number,
    name: string,
    cover: string
}
export interface AuthorWithCover {
    id: number;
    name: string;
    cover: string;
}

export interface Rating {
    avg: number,
    user: number
}

export interface Book {
    id: number;
    title: string;
    cover: string;
    description: string;
    status: BookStatus;
    tags: Tag[];
    genres: Genre[];
    authors: Author[];
    chapters_count: number;
    series_id: number | null;
    rating?: Rating;
}

export interface CreateBookFields {
    title: string;
    description: string;
    status: number;
    tags: number[];
    genres: number[];
    authors: number[];
    series_id?: number;
}

export interface UpdateBookFields {
    title?: string;
    description?: string;
    status?: number;
    series_id?: number | null;
    tags_to_add?: number[];
    tags_to_delete?: number[];
    genres_to_add?: number[];
    genres_to_delete?: number[];
    authors_to_add?: number[];
    authors_to_delete?: number[];
}

export interface ChapterSchema {
    id: number;
    index: number;
    name: string;
    book_id: number;
    created_at: string;
}

export interface ChapterFullSchema {
    id: number,
    index: number,
    name: string,
    content: any,
    book_id: number,
    created_at: string,
}