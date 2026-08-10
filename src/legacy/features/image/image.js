(() => {
  // ../src/legacy/features/image/image.ts
  (() => {
    const PLACEHOLDER_SVG_URI = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCA0MDAgMjAwIiB3aWR0aD0iMTAwJSIgaGVpZ2h0PSIxMDAlIj48cmVjdCB3aWR0aD0iMzk4IiBoZWlnaHQ9IjE5OCIgeD0iMSIgeT0iMSIgcng9IjgiIGZpbGw9IiNmOGZhZmMiIHN0cm9rZT0iI2NiZDVlMSIgc3Ryb2tlLWRhc2hhcnJheT0iNiA0Ii8+PGcgZmlsbD0ibm9uZSIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIj48cmVjdCB3aWR0aD0iODAiIGhlaWdodD0iNjAiIHg9IjE2MCIgeT0iNTIiIHJ4PSI2IiBmaWxsPSIjZmZmIiBzdHJva2U9IiM5NGEzYjgiIHN0cm9rZS13aWR0aD0iMiIvPjxjaXJjbGUgY3g9IjE4MCIgY3k9IjcxIiByPSI1IiBmaWxsPSIjY2JkNWUxIi8+PHBhdGggZD0ibTE2NSAxMDcgMTgtMjAgMTggMTYgMTItMTAgMjIgMTQiIHN0cm9rZT0iIzk0YTNiOCIgc3Ryb2tlLXdpZHRoPSIyIi8+PGNpcmNsZSBjeD0iMjI1IiBjeT0iNjIiIHI9IjkiIGZpbGw9IiNmMWY1ZjkiIHN0cm9rZT0iIzk0YTNiOCIgc3Ryb2tlLXdpZHRoPSIxLjUiLz48cGF0aCBkPSJtMjIxIDY2IDgtOCIgc3Ryb2tlPSIjZTExZDQ4IiBzdHJva2Utd2lkdGg9IjIiLz48L2c+PHRleHQgeD0iMjAwIiB5PSIxNDIiIGZpbGw9IiM2NDc0OGIiIGZvbnQtZmFtaWx5PSItYXBwbGUtc3lzdGVtLEJsaW5rTWFjU3lzdGVtRm9udCwnU2Vnb2UgVUknLFJvYm90byxzYW5zLXNlcmlmIiBmb250LXNpemU9IjEzIiBmb250LXdlaWdodD0iNjAwIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj5JbWFnZSB1bmF2YWlsYWJsZTwvdGV4dD48L3N2Zz4=";
    function handleLightboxKeydown(e) {
      if (e.key === "Escape") {
        closeLightbox();
      }
    }
    function openLightbox(imgSrc) {
      const rawLbImg = document.getElementById("lb-img");
      const lbImg = rawLbImg instanceof HTMLImageElement ? rawLbImg : null;
      const lightbox = document.getElementById("lightbox");
      if (lbImg && lightbox) {
        lbImg.src = imgSrc;
        lightbox.classList.add("active");
        document.addEventListener("keydown", handleLightboxKeydown);
      }
    }
    function closeLightbox() {
      const lightbox = document.getElementById("lightbox");
      if (lightbox) {
        lightbox.classList.remove("active");
      }
      document.removeEventListener("keydown", handleLightboxKeydown);
    }
    function applyImageFallback(img) {
      if (img.dataset["d2fFallback"] === "true") {
        return;
      }
      img.dataset["d2fFallback"] = "true";
      img.classList.add("img-fallback");
      img.src = PLACEHOLDER_SVG_URI;
    }
    function handleImageError(e) {
      const target = e.target;
      if (target instanceof HTMLImageElement) {
        applyImageFallback(target);
      }
    }
    function handleDocumentClick(e) {
      const target = e.target;
      if (target instanceof Element) {
        const lb = target.closest("#lightbox");
        if (lb) {
          const lbImg = target.closest("#lb-img");
          if (!lbImg) {
            closeLightbox();
          }
          return;
        }
        const imgEl = target.closest(".doc-body img");
        if (imgEl instanceof HTMLImageElement) {
          if (imgEl.classList.contains("img-fallback") || imgEl.dataset["d2fFallback"] === "true") {
            return;
          }
          e.stopPropagation();
          openLightbox(imgEl.src);
        }
      }
    }
    function checkExistingImages() {
      const images = document.querySelectorAll(".doc-body img, .img-item img");
      for (let i = 0; i < images.length; i++) {
        const img = images[i];
        if (img && img.complete && img.naturalWidth === 0 && img.src !== "") {
          applyImageFallback(img);
        }
      }
    }
    window.d2f_image = {
      open: openLightbox,
      close: closeLightbox,
      fallback: applyImageFallback
    };
    function init() {
      document.addEventListener("click", handleDocumentClick);
      document.addEventListener("error", handleImageError, true);
      checkExistingImages();
    }
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  })();
})();
