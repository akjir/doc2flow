export function handleLightboxKeydown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
        closeLightbox();
    }
}

export function openLightbox(imgSrc: string): void {
    const rawLbImg = document.getElementById('lb-img');
    const lbImg = rawLbImg instanceof HTMLImageElement ? rawLbImg : null;
    const lightbox = document.getElementById('lightbox');
    if (lbImg && lightbox) {
        lbImg.src = imgSrc;
        lightbox.classList.add('active');
        document.addEventListener('keydown', handleLightboxKeydown);
    }
}

export function closeLightbox(): void {
    const lightbox = document.getElementById('lightbox');
    if (lightbox) {
        lightbox.classList.remove('active');
    }
    document.removeEventListener('keydown', handleLightboxKeydown);
}

if (typeof window !== 'undefined') {
    document.addEventListener('DOMContentLoaded', () => {
        document.addEventListener('click', (e: MouseEvent) => {
            const target = e.target;
            if (target instanceof Element) {
                const img = target.closest<HTMLImageElement>('.doc-body img');
                if (img) {
                    e.stopPropagation();
                    openLightbox(img.src);
                }
            }
        });
    });
}
