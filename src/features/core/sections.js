(() => {
  const setSectionCollapseState = (secEl, isCollapsed) => {
    const body = secEl.querySelector('.section-body');
    const header = secEl.querySelector('.section-header');
    if (!body) return;

    body.classList.toggle('collapsed', isCollapsed);
    if (header) {
      header.setAttribute('aria-expanded', isCollapsed ? 'false' : 'true');
      const toggler = header.querySelector('.section-toggler');
      if (toggler) {
        toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
      }
    }
  };

  const updateEmptySections = () => {
    document.querySelectorAll('.section').forEach((sec) => {
      const header = sec.querySelector('.section-header');
      const body = sec.querySelector('.section-body');
      if (header && body && body.children.length === 0 && body.innerHTML.trim() === '') {
        header.classList.add('section-no-toggle');
        header.removeAttribute('role');
        header.removeAttribute('tabindex');
        header.removeAttribute('aria-expanded');
      }
    });
  };

  const toggleSection = (target, onSave) => {
    let header = null;

    if (typeof target === 'string') {
      const sec = document.getElementById(target);
      if (sec) {
        header = sec.querySelector('.section-header');
      }
    } else if (target instanceof HTMLElement) {
      header = target.closest('.section-header');
    }

    if (!header || header.classList.contains('section-no-toggle')) {
      return;
    }

    const section = header.closest('.section');
    const body = section ? section.querySelector('.section-body') : null;

    if (body && (body.children.length > 0 || body.innerHTML.trim() !== '')) {
      const isCollapsed = !body.classList.contains('collapsed');
      if (section) {
        setSectionCollapseState(section, isCollapsed);
      } else {
        body.classList.toggle('collapsed', isCollapsed);
        header.setAttribute('aria-expanded', isCollapsed ? 'false' : 'true');
        const toggler = header.querySelector('.section-toggler');
        if (toggler) {
          toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
        }
      }
      if (typeof onSave === 'function') {
        onSave();
      }
    }
  };

  const saveSections = () => {
    const sections = {};
    document.querySelectorAll('.section').forEach((sec, index) => {
      const body = sec.querySelector('.section-body');
      if (body) {
        const key = sec.id || `sec_${index}`;
        sections[key] = body.classList.contains('collapsed');
      }
    });
    return { sections };
  };

  const loadSections = (state) => {
    const sectionsData = state?.sections;
    const isRecord = window.d2f?.utils?.isRecord;
    const isValid = typeof isRecord === 'function'
      ? isRecord(sectionsData)
      : (typeof sectionsData === 'object' && sectionsData !== null && !Array.isArray(sectionsData));
    if (isValid) {
      document.querySelectorAll('.section').forEach((sec, index) => {
        const key = sec.id || `sec_${index}`;
        const shouldCollapse = sectionsData[key];
        if (typeof shouldCollapse === 'boolean') {
          setSectionCollapseState(sec, shouldCollapse);
        }
      });
    }
  };

  const resetSections = () => {
    document.querySelectorAll('.section').forEach((sec) => {
      setSectionCollapseState(sec, false);
    });
  };

  window.d2f = window.d2f || {};
  window.d2f.sections = {
    toggleSection,
    setSectionCollapseState,
    updateEmptySections,
    resetSections,
  };

  if (typeof window !== 'undefined') {
    if (window.d2f.storage?.registerSaveHandler) {
      window.d2f.storage.registerSaveHandler(saveSections);
    }
    if (window.d2f.storage?.registerLoadHandler) {
      window.d2f.storage.registerLoadHandler(loadSections);
    }
    if (window.d2f.core?.registerResetHandler) {
      window.d2f.core.registerResetHandler(resetSections);
    }

    if (document.readyState === 'loading') {
      document.addEventListener('DOMContentLoaded', () => {
        updateEmptySections();
      });
    } else {
      updateEmptySections();
    }

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        const target = e.target;
        if (target instanceof Element) {
          const header = target.closest('.section-header');
          if (header && !header.classList.contains('section-no-toggle')) {
            e.preventDefault();
            toggleSection(header, () => window.d2f.storage?.saveState?.());
          }
        }
      }
    });

    document.addEventListener('click', (e) => {
      const target = e.target;
      if (!(target instanceof Element)) return;

      const header = target.closest('.section-header');
      if (header && !header.classList.contains('section-no-toggle')) {
        toggleSection(header, () => window.d2f.storage?.saveState?.());
      }
    });
  }
})();
