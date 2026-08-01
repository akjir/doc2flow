window.exportPDF = () => window.d2f.export.exportPDF();
window.saveDocumentState = () => window.d2f.export.saveDocumentState();
window.resetAll = () => window.d2f.core.resetAll();

(() => {
    'use strict';

    const saveStateDebounced = window.d2f.utils.debounce(() => window.d2f.storage.saveState(), 300);

    document.addEventListener('DOMContentLoaded', () => {
        // Keyboard navigation delegation for section toggles
        document.addEventListener('keydown', (e: KeyboardEvent) => {
            if (e.key === 'Enter' || e.key === ' ') {
                const target = e.target;
                if (target instanceof Element) {
                    const sh = target.closest<HTMLElement>('.sh');
                    if (sh && !sh.classList.contains('no-toggle')) {
                        e.preventDefault();
                        window.d2f.collapse.toggleSection(sh, () => window.d2f.storage.saveState());
                    }
                }
            }
        });

        // Global click delegation
        document.addEventListener('click', (e: MouseEvent) => {
            const target = e.target;
            if (!(target instanceof Element)) return;

            const sh = target.closest<HTMLElement>('.sh');
            if (sh && !sh.classList.contains('no-toggle')) {
                window.d2f.collapse.toggleSection(sh, () => window.d2f.storage.saveState());
                return;
            }

            const commentBtn = target.closest<HTMLElement>('.item-comment-icon');
            if (commentBtn) {
                const checkItem = commentBtn.closest<HTMLElement>('.check-item');
                if (checkItem) {
                    const res = window.d2f.comments.getOrCreateCommentBox(checkItem);
                    if (res?.input) {
                        res.input.focus();
                    }
                }
                return;
            }

            const commentDelBtn = target.closest<HTMLElement>('.item-comment-del');
            if (commentDelBtn) {
                const box = commentDelBtn.closest<HTMLElement>('.item-comment-box');
                if (box) {
                    box.remove();
                    window.d2f.storage.saveState();
                }
                return;
            }

            const checkItem = target.closest<HTMLElement>('.check-item');
            if (checkItem) {
                if (target.tagName === 'A' || target.tagName === 'IMG' || target.closest('.item-comment-box')) {
                    return;
                }

                const cb = checkItem.querySelector<HTMLInputElement>('input[type="checkbox"]');
                if (cb) {
                    if (target !== cb && !target.closest('label')) {
                        cb.checked = !cb.checked;
                    }
                    window.d2f.tasks?.styleItem(cb);
                    window.d2f.tasks?.updateProgress();
                    window.d2f.storage.saveState();
                } else if (checkItem.classList.contains('text-item') || checkItem.classList.contains('simple-item')) {
                    checkItem.classList.toggle('checked');
                    window.d2f.storage.saveState();
                }
            }
        });

        const linkedIds: readonly string[] = ['f_info_agent', 'f_sign_agent', 'f_info_date', 'f_sign_date'];
        const handleInputOrChange = (e: Event): void => {
            const target = e.target;
            if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement)) return;

            if (target.classList.contains('persistent-field')) {
                saveStateDebounced();
            }

            if (target instanceof HTMLTextAreaElement && target.classList.contains('item-comment-input')) {
                target.textContent = target.value;
                target.setAttribute('value', target.value);
                saveStateDebounced();
            }

            if (target instanceof HTMLInputElement) {
                if (target.id && linkedIds.includes(target.id)) {
                    if (target.id.toLowerCase().includes('date')) {
                        window.d2f.fields.checkDateShortcut(target);
                    }
                    window.d2f.fields.syncLinkedFields(target);
                    saveStateDebounced();
                } else if (target.matches('input[id*="date"], input[name*="date"], input.date-field')) {
                    window.d2f.fields.checkDateShortcut(target);
                    saveStateDebounced();
                }
            }
        };

        document.addEventListener('input', handleInputOrChange);
        document.addEventListener('change', handleInputOrChange);

        // Search Toolbar Listeners
        const searchToggleBtn = document.getElementById('search-toggle-btn');
        if (searchToggleBtn) {
            searchToggleBtn.addEventListener('click', () => window.d2f.search.toggleSearchToolbar());
        }

        const rawSearchInput = document.getElementById('search-input');
        const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
        if (searchInput) {
            searchInput.addEventListener('input', () => window.d2f.search.performSearchAndFilter());
        }

        const searchClearBtn = document.getElementById('search-clear-btn');
        if (searchClearBtn) {
            searchClearBtn.addEventListener('click', () => {
                if (searchInput) {
                    searchInput.value = '';
                    searchInput.focus();
                }
                window.d2f.search.performSearchAndFilter(() => window.d2f.storage.saveState());
            });
        }

        document.addEventListener('keydown', (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
                e.preventDefault();
                window.d2f.search.toggleSearchToolbar(true);
            } else if (e.key === 'Escape') {
                const toolbar = document.getElementById('search-toolbar');
                if (toolbar && !toolbar.classList.contains('hidden')) {
                    e.preventDefault();
                    window.d2f.search.toggleSearchToolbar(false);
                }
            }
        });

        // Initialize Application
        window.d2f.collapse.updateEmptySections();
        window.d2f.storage.loadState();
        window.d2f.fields.syncLinkedFields();
        window.d2f.tasks?.updateProgress();
        window.d2f.search.performSearchAndFilter();
    });
})();

