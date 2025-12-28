/**
 * Song Cache Service
 * Uses IndexedDB to store downloaded songs locally in the browser
 */

const DB_NAME = 'neuro-music-cache';
const DB_VERSION = 1;
const STORE_NAME = 'songs';

interface CachedSong {
  id: string;          // song id
  youtubeId: string;   // youtube id for deduplication
  blob: Blob;          // audio data as OGG
  size: number;        // file size in bytes
  cachedAt: number;    // timestamp when cached
}

class SongCacheService {
  private db: IDBDatabase | null = null;
  private initPromise: Promise<void> | null = null;

  /**
   * Initialize IndexedDB
   */
  async init(): Promise<void> {
    if (this.db) return;
    if (this.initPromise) return this.initPromise;

    this.initPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);

      request.onerror = () => {
        console.error('❌ Failed to open IndexedDB:', request.error);
        reject(request.error);
      };

      request.onsuccess = () => {
        this.db = request.result;
        console.log('✅ IndexedDB initialized for song cache');
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = (event.target as IDBOpenDBRequest).result;
        
        // Create songs store if it doesn't exist
        if (!db.objectStoreNames.contains(STORE_NAME)) {
          const store = db.createObjectStore(STORE_NAME, { keyPath: 'id' });
          store.createIndex('youtubeId', 'youtubeId', { unique: false });
          store.createIndex('cachedAt', 'cachedAt', { unique: false });
          console.log('📦 Created songs object store');
        }
      };
    });

    return this.initPromise;
  }

  /**
   * Check if a song is cached
   */
  async has(songId: string): Promise<boolean> {
    await this.init();
    if (!this.db) return false;

    return new Promise((resolve) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getKey(songId);

      request.onsuccess = () => resolve(request.result !== undefined);
      request.onerror = () => resolve(false);
    });
  }

  /**
   * Get a cached song
   */
  async get(songId: string): Promise<Blob | null> {
    await this.init();
    if (!this.db) return null;

    return new Promise((resolve) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.get(songId);

      request.onsuccess = () => {
        const cached = request.result as CachedSong | undefined;
        if (cached) {
          console.log(`🎵 Cache HIT: ${songId} (${(cached.size / 1024 / 1024).toFixed(2)} MB)`);
          resolve(cached.blob);
        } else {
          console.log(`🎵 Cache MISS: ${songId}`);
          resolve(null);
        }
      };
      request.onerror = () => resolve(null);
    });
  }

  /**
   * Store a song in cache
   */
  async put(songId: string, youtubeId: string, blob: Blob): Promise<void> {
    await this.init();
    if (!this.db) return;

    const cached: CachedSong = {
      id: songId,
      youtubeId,
      blob,
      size: blob.size,
      cachedAt: Date.now(),
    };

    return new Promise((resolve, reject) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.put(cached);

      request.onsuccess = () => {
        console.log(`💾 Cached song: ${songId} (${(blob.size / 1024 / 1024).toFixed(2)} MB)`);
        resolve();
      };
      request.onerror = () => {
        console.error('❌ Failed to cache song:', request.error);
        reject(request.error);
      };
    });
  }

  /**
   * Remove a song from cache
   */
  async remove(songId: string): Promise<void> {
    await this.init();
    if (!this.db) return;

    return new Promise((resolve) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.delete(songId);

      request.onsuccess = () => {
        console.log(`🗑️ Removed from cache: ${songId}`);
        resolve();
      };
      request.onerror = () => resolve();
    });
  }

  /**
   * Get total cache size in bytes
   */
  async getTotalSize(): Promise<number> {
    await this.init();
    if (!this.db) return 0;

    return new Promise((resolve) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAll();

      request.onsuccess = () => {
        const songs = request.result as CachedSong[];
        const total = songs.reduce((sum, s) => sum + s.size, 0);
        resolve(total);
      };
      request.onerror = () => resolve(0);
    });
  }

  /**
   * Get number of cached songs
   */
  async getCount(): Promise<number> {
    await this.init();
    if (!this.db) return 0;

    return new Promise((resolve) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.count();

      request.onsuccess = () => resolve(request.result);
      request.onerror = () => resolve(0);
    });
  }

  /**
   * Clear all cached songs
   */
  async clear(): Promise<void> {
    await this.init();
    if (!this.db) return;

    return new Promise((resolve) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readwrite');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.clear();

      request.onsuccess = () => {
        console.log('🗑️ Cache cleared');
        resolve();
      };
      request.onerror = () => resolve();
    });
  }

  /**
   * Get all cached song IDs
   */
  async getAllIds(): Promise<string[]> {
    await this.init();
    if (!this.db) return [];

    return new Promise((resolve) => {
      const transaction = this.db!.transaction(STORE_NAME, 'readonly');
      const store = transaction.objectStore(STORE_NAME);
      const request = store.getAllKeys();

      request.onsuccess = () => resolve(request.result as string[]);
      request.onerror = () => resolve([]);
    });
  }
}

// Singleton instance
export const songCache = new SongCacheService();
