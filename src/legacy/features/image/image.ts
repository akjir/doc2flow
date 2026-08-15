interface Window {
    d2f_image: {
        readonly open: (src: string) => void;
        readonly close: () => void;
        readonly fallback: (img: HTMLImageElement) => void;
    };
}

(() => {
    const PLACEHOLDER_SVG_URI =
        'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MDAgMjAwIiB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjIwMCIgcng9IjgiIGZpbGw9IiNmOGZhZmMiLz48ZyBmaWxsPSJub25lIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxyZWN0IHdpZHRoPSI4MCIgaGVpZ2h0PSI2MCIgeD0iMTYwIiB5PSI1MiIgcng9IjYiIGZpbGw9IiNmZmYiIHN0cm9rZT0iIzk0YTNiOCIgc3Ryb2tlLXdpZHRoPSIyIi8+PGNpcmNsZSBjeD0iMTgwIiBjeT0iNzEiIHI9IjUiIGZpbGw9IiNjYmQ1ZTEiLz48cGF0aCBkPSJtMTY1IDEwNyAxOC0yMCAxOCAxNiAxMi0xMCAyMiAxNCIgc3Ryb2tlPSIjOTRhM2I4IiBzdHJva2Utd2lkdGg9IjIiLz48Y2lyY2xlIGN4PSIyMjUiIGN5PSI2MiIgcj0iOSIgZmlsbD0iI2Y4ZmFmYyIgc3Ryb2tlPSIjOTRhM2I4IiBzdHJva2Utd2lkdGg9IjEuNSIvPjxwYXRoIGQ9Im0yMjEgNjYgOC04IiBzdHJva2U9IiNlMTFkNDgiIHN0cm9rZS13aWR0aD0iMiIvPjwvZz48dGV4dCB4PSIyMDAiIHk9IjE0MiIgZmlsbD0iIzY0NzQ4YiIgZm9udC1mYW1pbHk9Ii1hcHBsZS1zeXN0ZW0sQmxpbmtNYWNTeXN0ZW1Gb250LCdTZWdvZSBVSScsUm9ib3RvLHNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iMTMiIGZvbnQtd2VpZ2h0PSI2MDAiIHRleHQtYW5jaG9yPSJtaWRkbGUiPkltYWdlIHVuYXZhaWxhYmxlPC90ZXh0Pjwvc3ZnPg==';

    function handleLightboxKeydown(e: KeyboardEvent): void {
        if (e.key === 'Escape') {
            closeLightbox();
        }
    }

    function openLightbox(imgSrc: string): void {
        const rawLbImg = document.getElementById('lb-img');
        const lbImg = rawLbImg instanceof HTMLImageElement ? rawLbImg : null;
        const lightbox = document.getElementById('lightbox');
        if (lbImg && lightbox) {
            lbImg.src = imgSrc;
            lightbox.classList.add('active');
            document.addEventListener('keydown', handleLightboxKeydown);
        }
    }

    function closeLightbox(): void {
        const lightbox = document.getElementById('lightbox');
        if (lightbox) {
            lightbox.classList.remove('active');
        }
        document.removeEventListener('keydown', handleLightboxKeydown);
    }

    function applyImageFallback(img: HTMLImageElement): void {
        if (img.dataset['d2fFallback'] === 'true') {
            return;
        }
        img.dataset['d2fFallback'] = 'true';
        img.classList.add('img-fallback');
        img.src = PLACEHOLDER_SVG_URI;
    }

    function handleImageError(e: Event): void {
        const target = e.target;
        if (target instanceof HTMLImageElement) {
            applyImageFallback(target);
        }
    }

    function handleDocumentClick(e: MouseEvent): void {
        const target = e.target;
        if (target instanceof Element) {
            const lb = target.closest('#lightbox');
            if (lb) {
                const lbImg = target.closest('#lb-img');
                if (!lbImg) {
                    closeLightbox();
                }
                return;
            }

            const imgEl = target.closest('.doc-body img');
            if (imgEl instanceof HTMLImageElement) {
                if (imgEl.classList.contains('img-fallback') || imgEl.dataset['d2fFallback'] === 'true') {
                    return;
                }
                e.stopPropagation();
                openLightbox(imgEl.src);
            }
        }
    }

    function checkExistingImages(): void {
        const images = document.querySelectorAll<HTMLImageElement>('.doc-body img, .img-item img');
        for (let i = 0; i < images.length; i++) {
            const img = images[i];
            if (img && img.complete && img.naturalWidth === 0 && img.src !== '') {
                applyImageFallback(img);
            }
        }
    }

    window.d2f_image = {
        open: openLightbox,
        close: closeLightbox,
        fallback: applyImageFallback,
    };

    function init(): void {
        document.addEventListener('click', handleDocumentClick);
        document.addEventListener('error', handleImageError, true);
        checkExistingImages();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
