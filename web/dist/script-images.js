"use strict";
(() => {
  // src/features/images.ts
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
  if (typeof window !== "undefined") {
    document.addEventListener("DOMContentLoaded", () => {
      document.addEventListener("click", (e) => {
        const target = e.target;
        if (target instanceof Element) {
          const img = target.closest(".doc-body img");
          if (img) {
            e.stopPropagation();
            openLightbox(img.src);
          }
        }
      });
    });
  }
})();
