/**
 * Doc2Flow Generic Checklist Logic
 */
(() => {
    'use strict';

    // Configuration & State
    const DOC_ID = window.D2F_DOC_ID || '';
    const rawFilename = window.location.pathname.split('/').pop() || 'index.html';
    const FILENAME = decodeURIComponent(rawFilename);
    const STATE_KEY = 'd2f_state_' + (DOC_ID ? (DOC_ID + '_') : '') + FILENAME;

    let preSearchCollapsedState = null;
    let lastMatchedSectionIds = new Set();

    // Utility: Debounce
    function debounce(func, wait) {
        let timeout;
        return function (...args) {
            clearTimeout(timeout);
            timeout = setTimeout(() => func.apply(this, args), wait);
        };
    }

    // Debounced SaveState for better typing performance
    const saveStateDebounced = debounce(saveState, 300);

    // Lightbox Control
    function handleLightboxKeydown(e) {
        if (e.key === 'Escape') closeLightbox();
    }

    function openLightbox(imgSrc) {
        const lbImg = document.getElementById('lb-img');
        const lightbox = document.getElementById('lightbox');
        if (lbImg && lightbox) {
            lbImg.src = imgSrc;
            lightbox.classList.add('active');
            document.addEventListener('keydown', handleLightboxKeydown);
        }
    }

    function closeLightbox() {
        const lightbox = document.getElementById('lightbox');
        if (lightbox) lightbox.classList.remove('active');
        document.removeEventListener('keydown', handleLightboxKeydown);
    }

    // Search Highlighting
    function removeHighlights(container) {
        if (!container) return;
        const highlights = container.querySelectorAll('mark.d2f-highlight');
        highlights.forEach(mark => {
            const parent = mark.parentNode;
            if (parent) {
                parent.replaceChild(document.createTextNode(mark.textContent), mark);
                parent.normalize();
            }
        });
    }

    function highlightTextNodes(container, query) {
        if (!container || !query) return;
        const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        const regex = new RegExp(escaped, 'gi');

        const walker = document.createTreeWalker(
            container,
            NodeFilter.SHOW_TEXT,
            {
                acceptNode: (node) => {
                    const parent = node.parentNode;
                    if (!parent) return NodeFilter.FILTER_REJECT;
                    const tag = parent.nodeName.toLowerCase();
                    if (['script', 'style', 'input', 'textarea', 'select', 'button'].includes(tag) ||
                        parent.classList.contains('d2f-highlight')) {
                        return NodeFilter.FILTER_REJECT;
                    }
                    return NodeFilter.FILTER_ACCEPT;
                }
            }
        );

        const textNodes = [];
        while (walker.nextNode()) textNodes.push(walker.currentNode);

        textNodes.forEach(textNode => {
            const val = textNode.nodeValue;
            if (!val || !regex.test(val)) return;

            regex.lastIndex = 0;
            const frag = document.createDocumentFragment();
            let lastIdx = 0;
            let match;

            while ((match = regex.exec(val)) !== null) {
                if (match.index > lastIdx) {
                    frag.appendChild(document.createTextNode(val.slice(lastIdx, match.index)));
                }
                const mark = document.createElement('mark');
                mark.className = 'd2f-highlight';
                mark.textContent = match[0];
                frag.appendChild(mark);
                lastIdx = regex.lastIndex;
                if (match[0].length === 0) break;
            }

            if (lastIdx < val.length) {
                frag.appendChild(document.createTextNode(val.slice(lastIdx)));
            }
            if (textNode.parentNode) {
                textNode.parentNode.replaceChild(frag, textNode);
            }
        });
    }

    // Search & Filter Operations
    function performSearchAndFilter() {
        const searchInput = document.getElementById('search-input');
        const searchCounter = document.getElementById('search-counter');
        const sections = document.querySelectorAll('.d2f-section, .section');
        if (!sections.length) return;

        const query = searchInput ? searchInput.value.trim() : '';
        const queryLower = query.toLowerCase();
        let visibleCount = 0;
        const totalCount = sections.length;

        if (queryLower.length > 0) {
            if (preSearchCollapsedState === null) {
                preSearchCollapsedState = new Map();
                sections.forEach(sec => {
                    const body = sec.querySelector('.sb');
                    if (body && sec.id) {
                        preSearchCollapsedState.set(sec.id, body.classList.contains('collapsed'));
                    }
                });
            }

            const currentMatchedIds = new Set();

            sections.forEach(sec => {
                removeHighlights(sec);
                const passesQuery = (sec.textContent || '').toLowerCase().includes(queryLower);

                if (passesQuery) {
                    sec.style.display = '';
                    visibleCount++;
                    if (sec.id) currentMatchedIds.add(sec.id);

                    const body = sec.querySelector('.sb');
                    if (body) {
                        highlightTextNodes(body, query);
                        if (body.classList.contains('collapsed')) {
                            body.classList.remove('collapsed');
                            const sh = sec.querySelector('.sh');
                            if (sh) {
                                sh.setAttribute('aria-expanded', 'true');
                                const toggler = sh.querySelector('.stog');
                                if (toggler) toggler.innerHTML = '&#9660;';
                            }
                        }
                    }
                } else {
                    sec.style.display = 'none';
                }
            });

            lastMatchedSectionIds = currentMatchedIds;
        } else {
            sections.forEach(sec => {
                removeHighlights(sec);
                sec.style.display = '';
                visibleCount++;

                const secId = sec.id;
                const body = sec.querySelector('.sb');
                const sh = sec.querySelector('.sh');

                if (body && secId && preSearchCollapsedState !== null) {
                    const wasMatched = lastMatchedSectionIds.has(secId);
                    const wasCollapsedBeforeSearch = preSearchCollapsedState.get(secId);

                    if (!wasMatched && wasCollapsedBeforeSearch === true) {
                        body.classList.add('collapsed');
                        if (sh) {
                            sh.setAttribute('aria-expanded', 'false');
                            const toggler = sh.querySelector('.stog');
                            if (toggler) toggler.innerHTML = '&#9650;';
                        }
                    }
                }
            });

            preSearchCollapsedState = null;
            lastMatchedSectionIds.clear();
            saveState();
        }

        const searchClearBtn = document.getElementById('search-clear-btn');
        if (searchClearBtn) {
            searchClearBtn.classList.toggle('hidden', query.length === 0);
        }

        if (searchCounter) {
            const i18n = window.D2F_I18N || {};
            const template = i18n.sections_visible || '{visible} / {total} sections visible';
            searchCounter.textContent = template
                .replace('{visible}', visibleCount)
                .replace('{total}', totalCount);
        }
    }

    function toggleSearchToolbar(show) {
        const toolbar = document.getElementById('search-toolbar');
        const toggleBtn = document.getElementById('search-toggle-btn');
        const input = document.getElementById('search-input');
        if (!toolbar) return;

        const shouldShow = typeof show === 'boolean' ? show : toolbar.classList.contains('hidden');

        if (shouldShow) {
            toolbar.classList.remove('hidden');
            if (toggleBtn) toggleBtn.classList.add('active');
            if (input) {
                input.focus();
                input.select();
            }
        } else {
            toolbar.classList.add('hidden');
            if (toggleBtn) toggleBtn.classList.remove('active');
            if (input) input.value = '';
            performSearchAndFilter();
        }
    }

    function updateEmptySections() {
        document.querySelectorAll('.d2f-section, .section').forEach(sec => {
            const sh = sec.querySelector('.sh');
            const body = sec.querySelector('.sb');
            if (sh && body && body.children.length === 0 && body.innerHTML.trim() === '') {
                sh.classList.add('no-toggle');
                sh.removeAttribute('role');
                sh.removeAttribute('tabindex');
                sh.removeAttribute('aria-expanded');
            }
        });
    }

    function toggleSection(headerElement) {
        if (typeof headerElement === 'string') {
            const sec = document.getElementById(headerElement);
            headerElement = sec ? sec.querySelector('.sh') : null;
        }
        if (!headerElement || headerElement.classList.contains('no-toggle')) return;
        const section = headerElement.closest('.section');
        const body = section ? section.querySelector('.sb') : null;

        if (body && (body.children.length > 0 || body.innerHTML.trim() !== '')) {
            const isCollapsed = body.classList.toggle('collapsed');
            headerElement.setAttribute('aria-expanded', isCollapsed ? 'false' : 'true');
            const toggler = headerElement.querySelector('.stog');
            if (toggler) toggler.innerHTML = isCollapsed ? '&#9650;' : '&#9660;';
            saveState();
        }
    }

    function autoExpandTextarea(el) {
        if (!el) return;
        el.style.height = 'auto';
        el.style.height = el.scrollHeight + 'px';
    }

    function getOrCreateCommentBox(checkItem, initialValue) {
        if (!checkItem) return null;
        let box = checkItem.querySelector('.item-comment-box');
        let input;

        if (!box) {
            box = document.createElement('div');
            box.className = 'item-comment-box';

            input = document.createElement('textarea');
            input.rows = 1;
            input.className = 'item-comment-input';
            const i18n = window.D2F_I18N || {};
            const commentLabel = i18n.comment_placeholder || 'Add a comment...';
            input.placeholder = commentLabel;
            input.setAttribute('aria-label', commentLabel);

            const delBtn = document.createElement('button');
            delBtn.type = 'button';
            delBtn.className = 'item-comment-del';
            delBtn.title = 'Delete comment';
            delBtn.setAttribute('aria-label', 'Delete comment');
            delBtn.innerHTML = '&#10006;';

            box.appendChild(input);
            box.appendChild(delBtn);
            checkItem.appendChild(box);
        } else {
            input = box.querySelector('.item-comment-input');
        }

        if (typeof initialValue === 'string') {
            input.value = initialValue;
            input.textContent = initialValue;
            input.setAttribute('value', initialValue);
        }

        autoExpandTextarea(input);
        return { box, input };
    }

    // Persistence Logic
    function saveState() {
        const state = {};
        document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
            state[cb.id || ('cb_' + index)] = cb.checked;
        });

        const textStates = {};
        document.querySelectorAll('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
            textStates[item.id || ('txt_' + index)] = item.classList.contains('checked');
        });

        const fields = {};
        document.querySelectorAll('input.persistent-field').forEach((input, index) => {
            fields[input.id || ('f_' + index)] = input.value;
        });

        const comments = {};
        document.querySelectorAll('.check-item').forEach((item, index) => {
            const input = item.querySelector('.item-comment-input');
            if (input && input.value.trim() !== '') {
                comments[item.id || ('item_' + index)] = input.value;
            }
        });

        const sections = {};
        document.querySelectorAll('.d2f-section, .section').forEach((sec, index) => {
            const body = sec.querySelector('.sb');
            if (body) {
                sections[sec.id || ('sec_' + index)] = body.classList.contains('collapsed');
            }
        });

        try {
            localStorage.setItem(STATE_KEY, JSON.stringify({
                checks: state,
                texts: textStates,
                fields: fields,
                comments: comments,
                sections: sections
            }));
        } catch (e) {
            console.warn('Failed to save state to localStorage', e);
        }
    }

    function loadState() {
        try {
            const raw = localStorage.getItem(STATE_KEY);
            if (!raw) return;

            const data = JSON.parse(raw);
            if (data.checks) {
                document.querySelectorAll('.check-item input[type="checkbox"]').forEach((cb, index) => {
                    const key = cb.id || ('cb_' + index);
                    if (data.checks[key] !== undefined) {
                        cb.checked = data.checks[key];
                        styleItem(cb);
                    }
                });
            }
            if (data.texts) {
                document.querySelectorAll('.check-item.text-item, .check-item.simple-item').forEach((item, index) => {
                    const key = item.id || ('txt_' + index);
                    if (data.texts[key] !== undefined) {
                        item.classList.toggle('checked', data.texts[key]);
                    }
                });
            }
            if (data.fields) {
                document.querySelectorAll('input.persistent-field').forEach((input, index) => {
                    const key = input.id || ('f_' + index);
                    if (data.fields[key] !== undefined) {
                        input.value = data.fields[key];
                    }
                });
            }
            if (data.comments) {
                document.querySelectorAll('.check-item').forEach((item, index) => {
                    const key = item.id || ('item_' + index);
                    if (data.comments[key] !== undefined) {
                        getOrCreateCommentBox(item, data.comments[key]);
                    }
                });
            }
            if (data.sections) {
                document.querySelectorAll('.d2f-section, .section').forEach((sec, index) => {
                    const key = sec.id || ('sec_' + index);
                    if (data.sections[key] === undefined) return;
                    const body = sec.querySelector('.sb');
                    const sh = sec.querySelector('.sh');
                    if (!body) return;
                    const shouldCollapse = data.sections[key];
                    body.classList.toggle('collapsed', shouldCollapse);
                    if (sh) {
                        sh.setAttribute('aria-expanded', shouldCollapse ? 'false' : 'true');
                        const toggler = sh.querySelector('.stog');
                        if (toggler) toggler.innerHTML = shouldCollapse ? '&#9650;' : '&#9660;';
                    }
                });
            }
            syncLinkedFields();
        } catch (e) {
            console.warn('Failed to load state from localStorage', e);
        }
    }

    function styleItem(cb) {
        const item = cb.closest('.check-item');
        if (item) item.classList.toggle('checked', cb.checked);
    }

    function updateProgress() {
        const i18n = window.D2F_I18N || {};
        const sections = document.querySelectorAll('.section');
        let total = 0, done = 0;

        const updates = Array.from(sections).map(sec => {
            const cbs = Array.from(sec.querySelectorAll('input[type="checkbox"]'));
            const badge = sec.querySelector('.sbadge');
            const count = cbs.length;
            const checkedCount = cbs.filter(c => c.checked).length;
            total += count;
            done += checkedCount;
            return { badge, count, checkedCount };
        });

        updates.forEach(({ badge, count, checkedCount }) => {
            if (!badge) return;
            if (count === 0) {
                badge.textContent = '';
                badge.style.display = 'none';
            } else {
                badge.style.display = '';
                badge.textContent = checkedCount + ' / ' + count;
                badge.className = 'sbadge' + (checkedCount === count ? ' done' : '');
            }
        });

        const pct = total ? Math.round((done / total) * 100) : 0;
        const pb = document.getElementById('pb');
        if (pb) {
            pb.style.width = pct + '%';
            if (pb.parentElement) pb.parentElement.setAttribute('aria-valuenow', pct);
        }

        const pt = document.getElementById('pt');
        if (pt) {
            const tmpl = i18n.progress_template || '{done} of {total} tasks completed ({pct}%)';
            pt.textContent = tmpl.replace('{done}', done).replace('{total}', total).replace('{pct}', pct);
        }

        const finishBox = document.getElementById('finish-box');
        const finishIcon = document.getElementById('finish-icon');
        const finishTitle = document.getElementById('finish-title');
        const btnPdf = document.getElementById('btn-pdf');

        if (finishBox) {
            finishBox.classList.remove('completed', 'pending', 'no-tasks');
            if (total === 0) {
                finishBox.classList.add('no-tasks');
                if (btnPdf) btnPdf.disabled = false;
            } else if (done < total) {
                finishBox.classList.add('pending');
                if (finishIcon) finishIcon.innerHTML = '&#x29D6;';
                if (finishTitle) finishTitle.textContent = i18n.setup_in_progress || 'Setup in Progress';
                if (btnPdf) btnPdf.disabled = true;
            } else {
                finishBox.classList.add('completed');
                if (finishIcon) finishIcon.innerHTML = '&#x2714;';
                if (finishTitle) finishTitle.textContent = i18n.setup_completed || 'Setup Completed';
                if (btnPdf) btnPdf.disabled = false;
            }
        }
    }

    function syncFieldPair(id1, id2, sourceInput) {
        const el1 = document.getElementById(id1);
        const el2 = document.getElementById(id2);
        if (!el1 || !el2) return;

        if (sourceInput === el1) {
            el2.value = el1.value;
        } else if (sourceInput === el2) {
            el1.value = el2.value;
        } else {
            if (el1.value && !el2.value) el2.value = el1.value;
            else if (el2.value && !el1.value) el1.value = el2.value;
            else if (el1.value) el2.value = el1.value;
        }
    }

    function syncLinkedFields(sourceInput) {
        syncFieldPair('f_info_agent', 'f_sign_agent', sourceInput);
        syncFieldPair('f_info_date', 'f_sign_date', sourceInput);
    }

    function formatDateFromTemplate(now, template) {
        if (!template || typeof template !== 'string') return null;

        const tokenMap = {
            'YYYY': String(now.getFullYear()),
            'YY': String(now.getFullYear()).slice(-2),
            'MM': String(now.getMonth() + 1).padStart(2, '0'),
            'DD': String(now.getDate()).padStart(2, '0'),
            'M': String(now.getMonth() + 1),
            'D': String(now.getDate())
        };

        const regex = /YYYY|YY|MM|DD|M|D/gi;
        let hasMatches = false;

        const formatted = template.replace(regex, (match) => {
            hasMatches = true;
            return tokenMap[match.toUpperCase()] || match;
        });

        return (hasMatches && !/[A-Za-z]/.test(formatted)) ? formatted : null;
    }

    function getTodayFormatted() {
        const i18n = window.D2F_I18N || {};
        const now = new Date();
        try {
            const fromTemplate = formatDateFromTemplate(now, i18n.date_placeholder);
            if (fromTemplate) return fromTemplate;
        } catch (e) {
            console.warn('Failed to format date', e);
        }
        return now.toLocaleDateString(navigator.language || undefined);
    }

    function checkDateShortcut(input) {
        if (!input || typeof input.value !== 'string') return false;
        if (input.value.trim().toLowerCase() === 'today') {
            input.value = getTodayFormatted();
            return true;
        }
        return false;
    }

    // Global Initialization & Event Listeners
    document.addEventListener('DOMContentLoaded', () => {
        // Keyboard navigation delegation
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' || e.key === ' ') {
                const sh = e.target.closest('.sh');
                if (sh && !sh.classList.contains('no-toggle')) {
                    e.preventDefault();
                    toggleSection(sh);
                }
            }
        });

        // Click delegation
        document.addEventListener('click', (e) => {
            const img = e.target.closest('.doc-body img');
            if (img) {
                e.stopPropagation();
                openLightbox(img.src);
                return;
            }

            const sh = e.target.closest('.sh');
            if (sh && !sh.classList.contains('no-toggle')) {
                toggleSection(sh);
                return;
            }

            const commentBtn = e.target.closest('.item-comment-icon');
            if (commentBtn) {
                const checkItem = commentBtn.closest('.check-item');
                if (checkItem) {
                    const res = getOrCreateCommentBox(checkItem);
                    if (res && res.input) res.input.focus();
                }
                return;
            }

            const commentDelBtn = e.target.closest('.item-comment-del');
            if (commentDelBtn) {
                const box = commentDelBtn.closest('.item-comment-box');
                if (box) {
                    box.remove();
                    saveState();
                }
                return;
            }

            const checkItem = e.target.closest('.check-item');
            if (checkItem) {
                if (e.target.tagName === 'A' || e.target.tagName === 'IMG' || e.target.closest('.item-comment-box')) return;

                const cb = checkItem.querySelector('input[type="checkbox"]');
                if (cb) {
                    if (e.target !== cb && !e.target.closest('label')) {
                        cb.checked = !cb.checked;
                    }
                    styleItem(cb);
                    updateProgress();
                    saveState();
                } else if (checkItem.classList.contains('text-item') || checkItem.classList.contains('simple-item')) {
                    checkItem.classList.toggle('checked');
                    saveState();
                }
            }
        });

        const linkedIds = ['f_info_agent', 'f_sign_agent', 'f_info_date', 'f_sign_date'];
        const handleInputOrChange = (e) => {
            const target = e.target;
            if (!target) return;

            if (target.classList.contains('persistent-field')) {
                saveStateDebounced();
            }

            if (target.classList.contains('item-comment-input')) {
                target.textContent = target.value;
                target.setAttribute('value', target.value);
                autoExpandTextarea(target);
                saveStateDebounced();
            }

            if (target.id && linkedIds.includes(target.id)) {
                if (target.id.toLowerCase().includes('date')) checkDateShortcut(target);
                syncLinkedFields(target);
                saveStateDebounced();
            } else if (target.matches('input[id*="date"], input[name*="date"], input.date-field')) {
                checkDateShortcut(target);
                saveStateDebounced();
            }
        };

        document.addEventListener('input', handleInputOrChange);
        document.addEventListener('change', handleInputOrChange);

        // Search Toolbar Listeners
        const searchToggleBtn = document.getElementById('search-toggle-btn');
        if (searchToggleBtn) {
            searchToggleBtn.addEventListener('click', () => toggleSearchToolbar());
        }

        const searchInput = document.getElementById('search-input');
        if (searchInput) {
            searchInput.addEventListener('input', debounce(performSearchAndFilter, 100));
        }

        const searchClearBtn = document.getElementById('search-clear-btn');
        if (searchClearBtn) {
            searchClearBtn.addEventListener('click', () => {
                if (searchInput) {
                    searchInput.value = '';
                    searchInput.focus();
                }
                performSearchAndFilter();
            });
        }

        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && (e.key === 'k' || e.key === 'K')) {
                e.preventDefault();
                toggleSearchToolbar(true);
            } else if (e.key === 'Escape') {
                const toolbar = document.getElementById('search-toolbar');
                if (toolbar && !toolbar.classList.contains('hidden')) {
                    e.preventDefault();
                    toggleSearchToolbar(false);
                }
            }
        });

        // Initialize Application
        updateEmptySections();
        loadState();
        syncLinkedFields();
        updateProgress();
        performSearchAndFilter();
    });
})();