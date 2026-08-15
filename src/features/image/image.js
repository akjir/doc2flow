(() => {
  const PLACEHOLDER_SVG_URI =
    'data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MDAgMjAwIiB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIj48cmVjdCB3aWR0aD0iMzk4IiBoZWlnaHQ9IjE5OCIgeD0iMSIgeT0iMSIgcng9IjgiIGZpbGw9IiNmOGZhZmMiIHN0cm9rZT0iI2NiZDVlMSIgc3Ryb2tlLWRhc2hhcnJheT0iNiA0Ii8+PGcgZmlsbD0ibm9uZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIj48cmVjdCB3aWR0aD0iODAiIGhlaWdodD0iNjAiIHg9IjE2MCIgeT0iNTIiIHJ4PSI2IiBmaWxsPSIjZmZmIiBzdHJva2U9IiM5NGEzYjgiIHN0cm9rZS13aWR0aD0iMiIvPjxjaXJjbGUgY3g9IjE4MCIgY3k9IjcxIiByPSI1IiBmaWxsPSIjY2JkNWUxIi8+PHBhdGggZD0ibTE2NSAxMDcgMTgtMjAgMTggMTYgMTItMTAgMjIgMTQiIHN0cm9rZT0iIzk0YTNiOCIgc3Ryb2tlLXdpZHRoPSIyIi8+PGNpcmNsZSBjeD0iMjI1IiBjeT0iNjIiIHI9IjkiIGZpbGw9IiNmMWY1ZjkiIHN0cm9rZT0iIzk0YTNiOCIgc3Ryb2tlLXdpZHRoPSIxLjUiLz48cGF0aCBkPSJtMjIxIDY2IDgtOCIgc3Ryb2tlPSIjZTExZDQ4IiBzdHJva2Utd2lkdGg9IjIiLz48L2c+PHRleHQgeD0iMjAwIiB5PSIxNDIiIGZpbGw9IiM2NDc0OGIiIGZvbnQtZmFtaWx5PSItYXBwbGUtc3lzdGVtLEJsaW5rTWFjU3lzdGVtRm9udCwnU2Vnb2UgVUknLFJvYm90byxzYW5zLXNlcmlmIiBmb250LXNpemU9IjEzIiBmb250LXdlaWdodD0iNjAwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj5JbWFnZSB1bmF2YWlsYWJsZTwvdGV4dD48L3N2Zz4=';

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
      lightbox.innerHTML =
        '<span class="image-lightbox-close">&times;</span><img id="image-lightbox-img" class="image-lightbox-img" src="" alt="" />';
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
