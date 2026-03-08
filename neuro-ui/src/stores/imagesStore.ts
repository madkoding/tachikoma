import { create } from 'zustand';
import {
  imagesApi,
  ImageDto,
  AlbumDto,
  ImageStyleDto,
  ImageSource,
  GenerateImageRequest,
  UpdateImageRequest,
  CreateAlbumRequest,
  UpdateAlbumRequest,
} from '../api/client';

// =============================================================================
// Types - Frontend models (camelCase)
// =============================================================================

export type { ImageSource };

export interface Album {
  id: string;
  name: string;
  description?: string;
  coverImageId?: string;
  imageCount: number;
  createdAt: Date;
  updatedAt: Date;
}

export interface Image {
  id: string;
  title: string;
  description?: string;
  url: string;
  thumbnailUrl?: string;
  width: number;
  height: number;
  sizeBytes: number;
  source: ImageSource;
  prompt?: string;
  negativePrompt?: string;
  model?: string;
  seed?: number;
  steps?: number;
  cfgScale?: number;
  tags: string[];
  albumId?: string;
  isFavorite: boolean;
  createdAt: Date;
}

export interface ImageStyle {
  id: string;
  name: string;
  description: string;
  promptModifier: string;
}

// =============================================================================
// Converters - API DTO (snake_case) to Frontend Model (camelCase)
// =============================================================================

function albumDtoToModel(dto: AlbumDto): Album {
  return {
    id: dto.id,
    name: dto.name,
    description: dto.description,
    coverImageId: dto.cover_image_id,
    imageCount: dto.image_count,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function imageDtoToModel(dto: ImageDto): Image {
  return {
    id: dto.id,
    title: dto.title,
    description: dto.description,
    url: dto.url,
    thumbnailUrl: dto.thumbnail_url,
    width: dto.width,
    height: dto.height,
    sizeBytes: dto.size_bytes,
    source: dto.source,
    prompt: dto.prompt,
    negativePrompt: dto.negative_prompt,
    model: dto.model,
    seed: dto.seed,
    steps: dto.steps,
    cfgScale: dto.cfg_scale,
    tags: dto.tags,
    albumId: dto.album_id,
    isFavorite: dto.is_favorite,
    createdAt: new Date(dto.created_at),
  };
}

function styleDtoToModel(dto: ImageStyleDto): ImageStyle {
  return {
    id: dto.id,
    name: dto.name,
    description: dto.description,
    promptModifier: dto.prompt_modifier,
  };
}

// =============================================================================
// Store Interface
// =============================================================================

interface ImagesState {
  // Data
  images: Image[];
  albums: Album[];
  styles: ImageStyle[];
  selectedImage: Image | null;
  selectedAlbumId: string | null;
  
  // UI State
  isLoading: boolean;
  isGenerating: boolean;
  error: string | null;
  showFavoritesOnly: boolean;
  viewMode: 'grid' | 'list';
  
  // Actions
  loadImages: (albumId?: string) => Promise<void>;
  loadAlbums: () => Promise<void>;
  loadStyles: () => Promise<void>;
  selectImage: (image: Image | null) => void;
  selectAlbum: (albumId: string | null) => void;
  setShowFavoritesOnly: (show: boolean) => void;
  setViewMode: (mode: 'grid' | 'list') => void;
  
  generateImage: (request: GenerateImageRequest) => Promise<Image>;
  updateImage: (id: string, data: UpdateImageRequest) => Promise<Image>;
  deleteImage: (id: string) => Promise<void>;
  toggleFavorite: (id: string) => Promise<Image>;
  
  createAlbum: (data: CreateAlbumRequest) => Promise<Album>;
  updateAlbum: (id: string, data: UpdateAlbumRequest) => Promise<Album>;
  deleteAlbum: (id: string) => Promise<void>;
  
  clearError: () => void;
}

// =============================================================================
// Store Implementation
// =============================================================================

export const useImagesStore = create<ImagesState>((set, get) => ({
  // Initial state
  images: [],
  albums: [],
  styles: [],
  selectedImage: null,
  selectedAlbumId: null,
  isLoading: false,
  isGenerating: false,
  error: null,
  showFavoritesOnly: false,
  viewMode: 'grid',

  // Load images with optional album filter
  loadImages: async (albumId?: string) => {
    set({ isLoading: true, error: null });
    try {
      const { showFavoritesOnly } = get();
      const dtos = await imagesApi.listImages(albumId, showFavoritesOnly);
      const images = dtos.map(imageDtoToModel);
      set({ images, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load images';
      set({ error: message, isLoading: false });
    }
  },

  // Load all albums
  loadAlbums: async () => {
    try {
      const dtos = await imagesApi.listAlbums();
      const albums = dtos.map(albumDtoToModel);
      set({ albums });
    } catch (error) {
      console.error('Failed to load albums:', error);
    }
  },

  // Load available styles
  loadStyles: async () => {
    try {
      const dtos = await imagesApi.getStyles();
      const styles = dtos.map(styleDtoToModel);
      set({ styles });
    } catch (error) {
      console.error('Failed to load styles:', error);
    }
  },

  selectImage: (image) => set({ selectedImage: image }),
  selectAlbum: (albumId) => {
    set({ selectedAlbumId: albumId });
    get().loadImages(albumId || undefined);
  },
  setShowFavoritesOnly: (show) => {
    set({ showFavoritesOnly: show });
    get().loadImages(get().selectedAlbumId || undefined);
  },
  setViewMode: (mode) => set({ viewMode: mode }),

  // Generate a new image
  generateImage: async (request) => {
    set({ isGenerating: true, error: null });
    try {
      const dto = await imagesApi.generateImage(request);
      const image = imageDtoToModel(dto);
      set((state) => ({
        images: [image, ...state.images],
        selectedImage: image,
        isGenerating: false,
      }));
      // Refresh album counts
      get().loadAlbums();
      return image;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to generate image';
      set({ error: message, isGenerating: false });
      throw error;
    }
  },

  // Update an image
  updateImage: async (id, data) => {
    try {
      const dto = await imagesApi.updateImage(id, data);
      const image = imageDtoToModel(dto);
      set((state) => ({
        images: state.images.map((i) => (i.id === id ? image : i)),
        selectedImage: state.selectedImage?.id === id ? image : state.selectedImage,
      }));
      // Refresh album counts if album changed
      if (data.album_id !== undefined) {
        get().loadAlbums();
      }
      return image;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update image';
      set({ error: message });
      throw error;
    }
  },

  // Delete an image
  deleteImage: async (id) => {
    try {
      await imagesApi.deleteImage(id);
      set((state) => ({
        images: state.images.filter((i) => i.id !== id),
        selectedImage: state.selectedImage?.id === id ? null : state.selectedImage,
      }));
      // Refresh album counts
      get().loadAlbums();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete image';
      set({ error: message });
      throw error;
    }
  },

  // Toggle favorite status
  toggleFavorite: async (id) => {
    try {
      const dto = await imagesApi.toggleFavorite(id);
      const image = imageDtoToModel(dto);
      set((state) => ({
        images: state.images.map((i) => (i.id === id ? image : i)),
        selectedImage: state.selectedImage?.id === id ? image : state.selectedImage,
      }));
      return image;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to toggle favorite';
      set({ error: message });
      throw error;
    }
  },

  // Create an album
  createAlbum: async (data) => {
    try {
      const dto = await imagesApi.createAlbum(data);
      const album = albumDtoToModel(dto);
      set((state) => ({
        albums: [...state.albums, album],
      }));
      return album;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to create album';
      set({ error: message });
      throw error;
    }
  },

  // Update an album
  updateAlbum: async (id, data) => {
    try {
      const dto = await imagesApi.updateAlbum(id, data);
      const album = albumDtoToModel(dto);
      set((state) => ({
        albums: state.albums.map((a) => (a.id === id ? album : a)),
      }));
      return album;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update album';
      set({ error: message });
      throw error;
    }
  },

  // Delete an album
  deleteAlbum: async (id) => {
    try {
      await imagesApi.deleteAlbum(id);
      set((state) => ({
        albums: state.albums.filter((a) => a.id !== id),
        selectedAlbumId: state.selectedAlbumId === id ? null : state.selectedAlbumId,
      }));
      // Reload images if we were viewing this album
      if (get().selectedAlbumId === id) {
        get().loadImages();
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete album';
      set({ error: message });
      throw error;
    }
  },

  clearError: () => set({ error: null }),
}));
