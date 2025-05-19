export interface BookPreview {
    id: string;
    title: string;
    thumbnail: string;
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

export interface Book {
    id: string;
    title: string;
    cover: string;
    description: string;
    status: BookStatus;
    tags: Tag[];
    genres: Genre[];
    authors: Author[];
    chapters_count: number;
    series_id: number | null;
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