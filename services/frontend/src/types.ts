export interface BookPreview {
    id: string;
    title: string;
    thumbnail: string;
}
  
export interface BookSearchResult extends BookPreview {
    status: string;
}

export interface Status {
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

export interface Author {
    id: number,
    name: string
}

export interface Book {
    id: string;
    title: string;
    cover: string;
    description: string;
    status: Status;
    tags: Tag[];
    genres: Genre[];
    authors: Author[];
    chapters_count: number;
}