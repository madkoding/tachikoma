import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useImagesStore, GeneratedImage, ImageAlbum, ImageGenerationRequest } from '../stores/imagesStore';
import TypewriterText from '../components/common/TypewriterText';

// =============================================================================
// Generate Image Modal
// =============================================================================

function GenerateModal({ onClose, onGenerate, isGenerating }: { onClose: () => void; onGenerate: (req: ImageGenerationRequest) => void; isGenerating: boolean }) {
  const { t } = useTranslation();
  const [prompt, setPrompt] = useState('');
  const [negativePrompt, setNegativePrompt] = useState('');
  const [width, setWidth] = useState(512);
  const [height, setHeight] = useState(512);
  const [steps, setSteps] = useState(30);
  const [guidanceScale, setGuidanceScale] = useState(7.5);

  const PRESETS = [
    { label: 'Square', w: 512, h: 512 },
    { label: 'Portrait', w: 512, h: 768 },
    { label: 'Landscape', w: 768, h: 512 },
    { label: 'HD', w: 1024, h: 1024 },
  ];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || isGenerating) return;
    onGenerate({ prompt: prompt.trim(), negative_prompt: negativePrompt.trim() || undefined, width, height, steps, guidance_scale: guidanceScale });
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-2xl shadow-2xl shadow-cyber-cyan/10 max-h-[90vh] overflow-auto">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between sticky top-0 bg-cyber-surface">
          <h2 className="text-lg font-cyber font-semibold text-cyber-cyan">🎨 {t('images.generate', 'Generate Image')}</h2>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('images.prompt', 'Prompt')} *</label>
            <textarea value={prompt} onChange={(e) => setPrompt(e.target.value)} rows={3} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm resize-none focus:border-cyber-cyan focus:outline-none" placeholder={t('images.promptPlaceholder', 'Describe the image you want to generate...')} autoFocus />
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('images.negativePrompt', 'Negative Prompt')}</label>
            <input type="text" value={negativePrompt} onChange={(e) => setNegativePrompt(e.target.value)} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" placeholder={t('images.negativePlaceholder', 'Elements to avoid...')} />
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">{t('images.size', 'Size')}</label>
            <div className="flex flex-wrap gap-2">
              {PRESETS.map((p) => (
                <button key={p.label} type="button" onClick={() => { setWidth(p.w); setHeight(p.h); }} className={`px-3 py-1.5 rounded-lg text-xs font-mono transition-all ${width === p.w && height === p.h ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'bg-cyber-bg text-cyber-cyan/50 border border-cyber-cyan/20 hover:border-cyber-cyan/40'}`}>
                  {p.label} ({p.w}×{p.h})
                </button>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('images.steps', 'Steps')}: {steps}</label>
              <input type="range" min={10} max={50} value={steps} onChange={(e) => setSteps(Number(e.target.value))} className="w-full accent-cyber-cyan" />
            </div>
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('images.guidance', 'Guidance')}: {guidanceScale}</label>
              <input type="range" min={1} max={20} step={0.5} value={guidanceScale} onChange={(e) => setGuidanceScale(Number(e.target.value))} className="w-full accent-cyber-cyan" />
            </div>
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <button type="button" onClick={onClose} className="px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm">{t('common.cancel', 'Cancel')}</button>
            <button type="submit" disabled={!prompt.trim() || isGenerating} className="cyber-button disabled:opacity-50 flex items-center gap-2">
              {isGenerating ? <><div className="animate-spin rounded-full h-4 w-4 border-b-2 border-current" /> {t('images.generating', 'Generating...')}</> : <>🎨 {t('images.generate', 'Generate')}</>}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// =============================================================================
// Image Detail Modal
// =============================================================================

function ImageDetailModal({ image, onClose, onDelete, onToggleFavorite }: { image: GeneratedImage; onClose: () => void; onDelete: () => void; onToggleFavorite: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="fixed inset-0 bg-black/90 backdrop-blur-sm flex items-center justify-center z-50 p-4" onClick={onClose}>
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-4xl shadow-2xl shadow-cyber-cyan/10 overflow-hidden max-h-[95vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
        <div className="p-3 border-b border-cyber-cyan/20 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <button onClick={onToggleFavorite} className="text-xl hover:scale-110 transition-transform">{image.isFavorite ? '❤️' : '🤍'}</button>
            <span className="text-xs font-mono text-cyber-cyan/50">{image.width}×{image.height}</span>
          </div>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>

        <div className="flex-1 overflow-auto p-4 flex items-center justify-center bg-black/50">
          <img src={image.url} alt={image.prompt} className="max-w-full max-h-[60vh] object-contain rounded-lg" />
        </div>

        <div className="p-4 border-t border-cyber-cyan/20 bg-cyber-surface">
          <p className="font-mono text-sm text-cyber-cyan mb-2">{image.prompt}</p>
          {image.negativePrompt && <p className="font-mono text-xs text-cyber-cyan/50 mb-2">❌ {image.negativePrompt}</p>}
          <div className="flex flex-wrap gap-2 text-xs font-mono text-cyber-cyan/40">
            <span>Steps: {image.steps}</span>
            <span>•</span>
            <span>Guidance: {image.guidanceScale}</span>
            <span>•</span>
            <span>{image.createdAt.toLocaleString()}</span>
          </div>
          <div className="flex justify-end gap-2 mt-4">
            <a href={image.url} download className="px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm flex items-center gap-2">⬇️ {t('common.download', 'Download')}</a>
            <button onClick={onDelete} className="px-4 py-2 text-red-400 hover:bg-red-500/10 rounded-lg transition-colors font-mono text-sm">🗑️ {t('common.delete', 'Delete')}</button>
          </div>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Album Sidebar
// =============================================================================

function AlbumSidebar({ albums, selectedAlbumId, showFavorites, onSelectAlbum, onShowFavorites, onCreateAlbum, onDeleteAlbum }: {
  albums: ImageAlbum[];
  selectedAlbumId: string | null;
  showFavorites: boolean;
  onSelectAlbum: (id: string | null) => void;
  onShowFavorites: (show: boolean) => void;
  onCreateAlbum: () => void;
  onDeleteAlbum: (id: string) => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="w-full md:w-56 lg:w-64 bg-cyber-surface border-r border-cyber-cyan/20 p-3 flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-cyber font-semibold text-cyber-cyan text-sm">{t('images.albums', 'Albums')}</h3>
        <button onClick={onCreateAlbum} className="p-1.5 text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">+</button>
      </div>

      <div className="space-y-1 flex-1 overflow-auto">
        <button onClick={() => { onSelectAlbum(null); onShowFavorites(false); }} className={`w-full px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${!selectedAlbumId && !showFavorites ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
          🖼️ {t('images.allImages', 'All Images')}
        </button>
        <button onClick={() => { onSelectAlbum(null); onShowFavorites(true); }} className={`w-full px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${showFavorites ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
          ❤️ {t('images.favorites', 'Favorites')}
        </button>

        {albums.length > 0 && <div className="border-t border-cyber-cyan/20 my-2" />}
        {albums.map((album) => (
          <div key={album.id} className="group flex items-center">
            <button onClick={() => { onSelectAlbum(album.id); onShowFavorites(false); }} className={`flex-1 px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${selectedAlbumId === album.id ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
              📁 {album.name}
            </button>
            <button onClick={() => onDeleteAlbum(album.id)} className="opacity-0 group-hover:opacity-100 p-1.5 text-cyber-cyan/40 hover:text-red-400 transition-all">🗑️</button>
          </div>
        ))}
      </div>
    </div>
  );
}

// =============================================================================
// Main Images Page
// =============================================================================

export default function ImagesPage() {
  const { t } = useTranslation();
  const { images, albums, selectedAlbumId, isLoading, isGenerating, error, loadImages, loadAlbums, selectAlbum, generateImage, deleteImage, toggleFavorite, createAlbum, deleteAlbum, clearError } = useImagesStore();

  const [showGenerateModal, setShowGenerateModal] = useState(false);
  const [selectedImage, setSelectedImage] = useState<GeneratedImage | null>(null);
  const [showSidebar, setShowSidebar] = useState(false);
  const [showFavorites, setShowFavorites] = useState(false);
  const [newAlbumName, setNewAlbumName] = useState('');
  const [showNewAlbumInput, setShowNewAlbumInput] = useState(false);

  useEffect(() => { loadImages(); loadAlbums(); }, [loadImages, loadAlbums]);

  const handleGenerate = async (req: ImageGenerationRequest) => {
    try { await generateImage(req); setShowGenerateModal(false); } catch (e) { console.error('Failed to generate image:', e); }
  };

  const handleDeleteImage = async () => {
    if (selectedImage && window.confirm(t('images.confirmDelete', 'Delete this image?'))) {
      try { await deleteImage(selectedImage.id); setSelectedImage(null); } catch (e) { console.error('Failed to delete image:', e); }
    }
  };

  const handleToggleFavorite = async () => {
    if (selectedImage) { try { await toggleFavorite(selectedImage.id); } catch (e) { console.error('Failed to toggle favorite:', e); } }
  };

  const handleCreateAlbum = async () => {
    if (showNewAlbumInput && newAlbumName.trim()) {
      try { await createAlbum({ name: newAlbumName.trim() }); setNewAlbumName(''); setShowNewAlbumInput(false); } catch (e) { console.error('Failed to create album:', e); }
    } else { setShowNewAlbumInput(true); }
  };

  const handleDeleteAlbum = async (id: string) => {
    if (window.confirm(t('images.confirmDeleteAlbum', 'Delete this album?'))) {
      try { await deleteAlbum(id); } catch (e) { console.error('Failed to delete album:', e); }
    }
  };

  // Filter images
  let filteredImages = images;
  if (showFavorites) filteredImages = images.filter(img => img.isFavorite);
  else if (selectedAlbumId) filteredImages = images.filter(img => img.albumId === selectedAlbumId);

  // Sort by date
  filteredImages = [...filteredImages].sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());

  return (
    <div className="h-full flex flex-col bg-cyber-bg overflow-hidden">
      <div className="fixed inset-0 pointer-events-none opacity-5" style={{ backgroundImage: 'linear-gradient(to right, cyan 1px, transparent 1px), linear-gradient(to bottom, cyan 1px, transparent 1px)', backgroundSize: '40px 40px' }} />

      <header className="p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 sm:gap-4">
            <button onClick={() => setShowSidebar(!showSidebar)} className="md:hidden p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" /></svg>
            </button>
            <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan"><TypewriterText text={t('nav.images', 'Images')} speed={30} /></h1>
          </div>
          <button onClick={() => setShowGenerateModal(true)} className="cyber-button text-sm flex items-center gap-2">
            🎨 <span className="hidden sm:inline">{t('images.generate', 'Generate')}</span>
          </button>
        </div>
      </header>

      {error && (
        <div className="mx-4 mt-4 p-3 bg-red-500/20 border border-red-500/50 text-red-400 rounded-lg flex justify-between items-center font-mono text-sm">
          <span>{error}</span><button onClick={clearError} className="text-red-400 hover:text-red-300">✕</button>
        </div>
      )}

      <div className="flex-1 flex overflow-hidden relative">
        {isLoading && !isGenerating && <div className="absolute inset-0 bg-cyber-bg/80 flex items-center justify-center z-20"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyber-cyan" /></div>}

        <div className={`${showSidebar ? 'flex' : 'hidden md:flex'} absolute md:relative inset-0 md:inset-auto z-10`}>
          <button onClick={() => setShowSidebar(false)} className="md:hidden absolute inset-0 bg-black/50" />
          <div className="relative">
            <button onClick={() => setShowSidebar(false)} className="md:hidden absolute top-2 right-2 z-10 p-2 text-cyber-cyan/70 hover:text-cyber-cyan">✕</button>
            <AlbumSidebar albums={albums} selectedAlbumId={selectedAlbumId} showFavorites={showFavorites} onSelectAlbum={(id) => { selectAlbum(id); setShowSidebar(false); }} onShowFavorites={(show) => { setShowFavorites(show); setShowSidebar(false); }} onCreateAlbum={handleCreateAlbum} onDeleteAlbum={handleDeleteAlbum} />
          </div>
        </div>

        {showNewAlbumInput && (
          <div className="absolute left-4 md:left-60 top-4 bg-cyber-surface border border-cyber-cyan/30 rounded-lg shadow-lg p-3 z-30">
            <input type="text" value={newAlbumName} onChange={(e) => setNewAlbumName(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && handleCreateAlbum()} placeholder={t('images.albumName', 'Album name')} className="px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm" autoFocus />
            <div className="flex gap-2 mt-2">
              <button onClick={handleCreateAlbum} className="px-3 py-1 bg-cyber-cyan text-cyber-bg rounded-lg text-sm font-mono">{t('common.create', 'Create')}</button>
              <button onClick={() => { setShowNewAlbumInput(false); setNewAlbumName(''); }} className="px-3 py-1 text-cyber-cyan/70 hover:text-cyber-cyan text-sm font-mono">{t('common.cancel', 'Cancel')}</button>
            </div>
          </div>
        )}

        <div className={`flex-1 overflow-auto p-3 sm:p-4 ${showSidebar ? 'hidden md:block' : 'block'}`}>
          {isGenerating && (
            <div className="mb-4 p-4 bg-cyber-surface border border-cyber-cyan/30 rounded-xl flex items-center gap-4">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyber-cyan" />
              <div>
                <p className="text-cyber-cyan font-mono text-sm">{t('images.generating', 'Generating image...')}</p>
                <p className="text-cyber-cyan/50 font-mono text-xs">{t('images.generatingDesc', 'This may take a moment')}</p>
              </div>
            </div>
          )}

          {filteredImages.length === 0 && !isGenerating ? (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <span className="text-5xl mb-4">🖼️</span>
              <p className="text-cyber-cyan/50 font-mono text-sm mb-4">{t('images.noImages', 'No images yet')}</p>
              <button onClick={() => setShowGenerateModal(true)} className="cyber-button">🎨 {t('images.generate', 'Generate Image')}</button>
            </div>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3 sm:gap-4">
              {filteredImages.map((img) => (
                <button key={img.id} onClick={() => setSelectedImage(img)} className="group relative aspect-square rounded-xl overflow-hidden border border-cyber-cyan/20 hover:border-cyber-cyan/50 transition-all hover:shadow-lg hover:shadow-cyber-cyan/20">
                  <img src={img.thumbnailUrl || img.url} alt={img.prompt} className="w-full h-full object-cover transition-transform group-hover:scale-105" />
                  <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
                    <div className="absolute bottom-0 left-0 right-0 p-2">
                      <p className="text-white text-xs font-mono truncate">{img.prompt}</p>
                    </div>
                  </div>
                  {img.isFavorite && <span className="absolute top-2 right-2 text-sm">❤️</span>}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {showGenerateModal && <GenerateModal onClose={() => setShowGenerateModal(false)} onGenerate={handleGenerate} isGenerating={isGenerating} />}
      {selectedImage && <ImageDetailModal image={selectedImage} onClose={() => setSelectedImage(null)} onDelete={handleDeleteImage} onToggleFavorite={handleToggleFavorite} />}
    </div>
  );
}
