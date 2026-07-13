import api from './client';

export type ImageSource = 'generated' | 'uploaded' | 'external';

export interface AlbumDto {
  id: string;
  name: string;
  description?: string;
  cover_image_id?: string;
  image_count: number;
  created_at: string;
  updated_at: string;
}

export interface ImageDto {
  id: string;
  title: string;
  description?: string;
  url: string;
  thumbnail_url?: string;
  width: number;
  height: number;
  size_bytes: number;
  source: ImageSource;
  prompt?: string;
  negative_prompt?: string;
  model?: string;
  seed?: number;
  steps?: number;
  cfg_scale?: number;
  tags: string[];
  album_id?: string;
  is_favorite: boolean;
  created_at: string;
}

export interface GenerateImageRequest {
  prompt: string;
  negative_prompt?: string;
  width?: number;
  height?: number;
  steps?: number;
  cfg_scale?: number;
  seed?: number;
  style?: string;
}

export interface ImageStyleDto {
  id: string;
  name: string;
  description: string;
  prompt_modifier: string;
}

export interface CreateAlbumRequest {
  name: string;
  description?: string;
}

export interface UpdateAlbumRequest {
  name?: string;
  description?: string;
  cover_image_id?: string;
}

export interface UpdateImageRequest {
  title?: string;
  description?: string;
  tags?: string[];
  album_id?: string;
  is_favorite?: boolean;
}

export const imagesApi = {
  listImages: async (albumId?: string, favoritesOnly = false): Promise<ImageDto[]> => {
    const response = await api.get<{ images: ImageDto[]; total: number; has_more: boolean }>('/images', {
      params: { album_id: albumId, favorites_only: favoritesOnly }
    });
    return response.data.images || [];
  },

  getImage: async (id: string): Promise<ImageDto> => {
    const response = await api.get<ImageDto>(`/images/${id}`);
    return response.data;
  },

  generateImage: async (request: GenerateImageRequest): Promise<ImageDto> => {
    const response = await api.post<ImageDto>('/images/generate', request);
    return response.data;
  },

  updateImage: async (id: string, request: UpdateImageRequest): Promise<ImageDto> => {
    const response = await api.patch<ImageDto>(`/images/${id}`, request);
    return response.data;
  },

  deleteImage: async (id: string): Promise<void> => {
    await api.delete(`/images/${id}`);
  },

  toggleFavorite: async (id: string): Promise<ImageDto> => {
    const response = await api.post<ImageDto>(`/images/${id}/favorite`);
    return response.data;
  },

  getStyles: async (): Promise<ImageStyleDto[]> => {
    const response = await api.get<{ styles: ImageStyleDto[] } | ImageStyleDto[]>('/images/styles');
    return Array.isArray(response.data) ? response.data : (response.data.styles || []);
  },

  listAlbums: async (): Promise<AlbumDto[]> => {
    const response = await api.get<{ albums: AlbumDto[] } | AlbumDto[]>('/images/albums');
    return Array.isArray(response.data) ? response.data : (response.data.albums || []);
  },

  getAlbum: async (id: string): Promise<AlbumDto> => {
    const response = await api.get<AlbumDto>(`/images/albums/${id}`);
    return response.data;
  },

  createAlbum: async (request: CreateAlbumRequest): Promise<AlbumDto> => {
    const response = await api.post<AlbumDto>('/images/albums', request);
    return response.data;
  },

  updateAlbum: async (id: string, request: UpdateAlbumRequest): Promise<AlbumDto> => {
    const response = await api.patch<AlbumDto>(`/images/albums/${id}`, request);
    return response.data;
  },

  deleteAlbum: async (id: string): Promise<void> => {
    await api.delete(`/images/albums/${id}`);
  },
};
