(() => {
  const PLACEHOLDER_SVG_URI =
    'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MDAgMjAwIiB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjIwMCIgcng9IjgiIGZpbGw9IiNmOGZhZmMiLz48ZyBmaWxsPSJub25lIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjxyZWN0IHdpZHRoPSI4MCIgaGVpZ2h0PSI2MCIgeD0iMTYwIiB5PSI1MiIgcng9IjYiIGZpbGw9IiNmZmYiIHN0cm9rZT0iIzk0YTNiOCIgc3Ryb2tlLXdpZHRoPSIyIi8+PGNpcmNsZSBjeD0iMTgwIiBjeT0iNzEiIHI9IjUiIGZpbGw9IiNjYmQ1ZTEiLz48cGF0aCBkPSJtMTY1IDEwNyAxOC0yMCAxOCAxNiAxMi0xMCAyMiAxNCIgc3Ryb2tlPSIjOTRhM2I4IiBzdHJva2Utd2lkdGg9IjIiLz48Y2lyY2xlIGN4PSIyMjUiIGN5PSI2MiIgcj0iOSIgZmlsbD0iI2Y4ZmFmYyIgc3Ryb2tlPSIjOTRhM2I4IiBzdHJva2Utd2lkdGg9IjEuNSIvPjxwYXRoIGQ9Im0yMjEgNjYgOC04IiBzdHJva2U9IiNlMTFkNDgiIHN0cm9rZS13aWR0aD0iMiIvPjwvZz48dGV4dCB4PSIyMDAiIHk9IjE0MiIgZmlsbD0iIzY0NzQ4YiIgZm9udC1mYW1pbHk9Ii1hcHBsZS1zeXN0ZW0sQmxpbmtNYWNTeXN0ZW1Gb250LCdTZWdvZSBVSScsUm9ib3RvLHNhbnMtc2VyaWYiIGZvbnQtc2l6ZT0iMTMiIGZvbnQtd2VpZ2h0PSI2MDAiIHRleHQtYW5jaG9yPSJtaWRkbGUiPkltYWdlIHVuYXZhaWxhYmxlPC90ZXh0Pjwvc3ZnPg==';

  const handleLightboxKeydown = (e) => {
    if (e.key === 'Escape') {
      closeLightbox();
    }
  };

  const getOrCreateLightbox = () => {
    let lightbox =
      document.getElementById('image-lightbox') ||
      document.getElementById('lightbox') ||
      document.querySelector('.image-lightbox');

    if (!lightbox) {
      lightbox = document.createElement('div');
      lightbox.id = 'image-lightbox';
      lightbox.className = 'image-lightbox';

      const closeBtn = document.createElement('span');
      closeBtn.className = 'image-lightbox-close';
      closeBtn.innerHTML = '&times;';

      const img = document.createElement('img');
      img.id = 'image-lightbox-img';
      img.className = 'image-lightbox-img';
      img.alt = '';

      lightbox.appendChild(closeBtn);
      lightbox.appendChild(img);
      document.body.appendChild(lightbox);
    }
    return lightbox;
  };

  const openLightbox = (imgSrc) => {
    const lightbox = getOrCreateLightbox();
    const lbImg =
      lightbox.querySelector('#image-lightbox-img') ||
      lightbox.querySelector('#lb-img') ||
      lightbox.querySelector('.image-lightbox-img') ||
      lightbox.querySelector('img');

    if (lbImg && lightbox) {
      lbImg.src = imgSrc;
      lightbox.classList.add('active');
      document.addEventListener('keydown', handleLightboxKeydown);
    }
  };

  const closeLightbox = () => {
    const lightbox =
      document.getElementById('image-lightbox') ||
      document.getElementById('lightbox') ||
      document.querySelector('.image-lightbox');

    if (lightbox) {
      lightbox.classList.remove('active');
      lightbox.classList.remove('image-lightbox-active');
    }
    document.removeEventListener('keydown', handleLightboxKeydown);
  };

  const applyImageFallback = (img) => {
    if (img.dataset.d2fFallback === 'true') {
      return;
    }
    img.dataset.d2fFallback = 'true';
    img.classList.add('image-fallback');
    img.classList.add('img-fallback');
    img.src = PLACEHOLDER_SVG_URI;
  };

  const handleImageError = (e) => {
    const { target } = e;
    if (target instanceof HTMLImageElement) {
      applyImageFallback(target);
    }
  };

  const handleDocumentClick = (e) => {
    const { target } = e;
    if (!(target instanceof Element)) {
      return;
    }

    const lightbox = target.closest('#image-lightbox, #lightbox, .image-lightbox');
    if (lightbox) {
      const lbImg = target.closest('#image-lightbox-img, #lb-img, .image-lightbox-img');
      if (!lbImg) {
        closeLightbox();
      }
      return;
    }

    const imgEl = target.closest('.image-item img, .doc-body img, .img-item img');
    if (imgEl instanceof HTMLImageElement) {
      if (
        imgEl.classList.contains('image-fallback') ||
        imgEl.classList.contains('img-fallback') ||
        imgEl.dataset.d2fFallback === 'true'
      ) {
        return;
      }
      e.stopPropagation();
      openLightbox(imgEl.src);
    }
  };

  const checkExistingImages = () => {
    const images = document.querySelectorAll('.image-item img, .doc-body img, .img-item img');
    images.forEach((img) => {
      if (img.complete && img.naturalWidth === 0 && img.src !== '') {
        applyImageFallback(img);
      }
    });
  };

  window.d2f = window.d2f || {};
  window.d2f.image = {
    open: openLightbox,
    close: closeLightbox,
    fallback: applyImageFallback,
  };

  const init = () => {
    document.addEventListener('click', handleDocumentClick);
    document.addEventListener('error', handleImageError, true);
    checkExistingImages();
  };

  if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', init);
    } else {
      init();
    }
  }
})();
