(() => {
  // ../src/features/image/image.ts
  (() => {
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
          e.stopPropagation();
          openLightbox(imgEl.src);
        }
      }
    }
    window.d2f_image = {
      open: openLightbox,
      close: closeLightbox
    };
    function init() {
      document.addEventListener("click", handleDocumentClick);
    }
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  })();
})();
