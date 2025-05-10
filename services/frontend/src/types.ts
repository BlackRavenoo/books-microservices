export interface BookPreview {
    id: string;
    title: string;
    thumbnail: string;
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
    name: string
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