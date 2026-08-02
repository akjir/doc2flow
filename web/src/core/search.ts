let preSearchCollapsedState: Map<string, boolean> | null = null;
let lastMatchedSectionIds: Set<string> = new Set();

function removeHighlights(container: HTMLElement | null): void {
    if (!container) return;
    const highlights = container.querySelectorAll<HTMLElement>('mark.d2f-highlight');
    highlights.forEach((mark) => {
        const parent = mark.parentNode;
        if (parent) {
            const textContent = mark.textContent ?? '';
            parent.replaceChild(document.createTextNode(textContent), mark);
            parent.normalize();
        }
    });
}

function highlightTextNodes(container: HTMLElement | null, query: string): void {
    if (!container || !query) return;
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(escaped, 'gi');

    const walker = document.createTreeWalker(
        container,
        NodeFilter.SHOW_TEXT,
        {
            acceptNode: (node: Node): number => {
                const parent = node.parentNode;
                if (!parent || !(parent instanceof HTMLElement)) return NodeFilter.FILTER_REJECT;
                const tag = parent.nodeName.toLowerCase();
                if (['script', 'style', 'input', 'textarea', 'select', 'button'].includes(tag) ||
                    parent.classList.contains('d2f-highlight')) {
                    return NodeFilter.FILTER_REJECT;
                }
                return NodeFilter.FILTER_ACCEPT;
            }
        }
    );

    const textNodes: Text[] = [];
    let currentNode = walker.nextNode();
    while (currentNode) {
        if (currentNode instanceof Text) {
            textNodes.push(currentNode);
        }
        currentNode = walker.nextNode();
    }

    textNodes.forEach((textNode) => {
        const val = textNode.nodeValue;
        if (!val || !regex.test(val)) return;

        regex.lastIndex = 0;
        const frag = document.createDocumentFragment();
        let lastIdx = 0;
        let match: RegExpExecArray | null = regex.exec(val);

        while (match !== null) {
            if (match.index > lastIdx) {
                frag.appendChild(document.createTextNode(val.slice(lastIdx, match.index)));
            }
            const mark = document.createElement('mark');
            mark.className = 'd2f-highlight';
            const matchedText = match[0] ?? '';
            mark.textContent = matchedText;
            frag.appendChild(mark);
            lastIdx = regex.lastIndex;
            if (matchedText.length === 0) break;
            match = regex.exec(val);
        }

        if (lastIdx < val.length) {
            frag.appendChild(document.createTextNode(val.slice(lastIdx)));
        }
        if (textNode.parentNode) {
            textNode.parentNode.replaceChild(frag, textNode);
        }
    });
}

function performSearchAndFilter(): void {
    const rawSearchInput = document.getElementById('search-input');
    const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
    const searchCounter = document.getElementById('search-counter');
    const sections = document.querySelectorAll<HTMLElement>('.section');
    if (sections.length === 0) return;

    const query = searchInput ? searchInput.value.trim() : '';
    const queryLower = query.toLowerCase();
    let visibleCount = 0;
    const totalCount = sections.length;

    if (queryLower.length > 0) {
        if (preSearchCollapsedState === null) {
            preSearchCollapsedState = new Map();
            sections.forEach((sec) => {
                const body = sec.querySelector<HTMLElement>('.sb');
                if (body && sec.id) {
                    preSearchCollapsedState?.set(sec.id, body.classList.contains('collapsed'));
                }
            });
        }

        const currentMatchedIds = new Set<string>();

        sections.forEach((sec) => {
            removeHighlights(sec);
            const passesQuery = (sec.textContent ?? '').toLowerCase().includes(queryLower);

            if (passesQuery) {
                sec.style.display = '';
                visibleCount++;
                if (sec.id) currentMatchedIds.add(sec.id);

                const body = sec.querySelector<HTMLElement>('.sb');
                if (body) {
                    highlightTextNodes(body, query);
                    if (body.classList.contains('collapsed')) {
                        body.classList.remove('collapsed');
                        const sh = sec.querySelector<HTMLElement>('.sh');
                        if (sh) {
                            sh.setAttribute('aria-expanded', 'true');
                            const toggler = sh.querySelector<HTMLElement>('.stog');
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
        sections.forEach((sec) => {
            removeHighlights(sec);
            sec.style.display = '';
            visibleCount++;

            const secId = sec.id;
            const body = sec.querySelector<HTMLElement>('.sb');
            const sh = sec.querySelector<HTMLElement>('.sh');

            if (body && secId && preSearchCollapsedState !== null) {
                const wasMatched = lastMatchedSectionIds.has(secId);
                const wasCollapsedBeforeSearch = preSearchCollapsedState.get(secId);

                if (!wasMatched && wasCollapsedBeforeSearch === true) {
                    body.classList.add('collapsed');
                    if (sh) {
                        sh.setAttribute('aria-expanded', 'false');
                        const toggler = sh.querySelector<HTMLElement>('.stog');
                        if (toggler) toggler.innerHTML = '&#9650;';
                    }
                }
            }
        });

        preSearchCollapsedState = null;
        lastMatchedSectionIds.clear();
    }

    const searchClearBtn = document.getElementById('search-clear-btn');
    if (searchClearBtn) {
        searchClearBtn.classList.toggle('hidden', query.length === 0);
    }

    if (searchCounter) {
        const i18n = window.d2f.lang.dictionary;
        const template = i18n.sections_visible ?? '{visible} / {total} sections visible';
        searchCounter.textContent = template
            .replace('{visible}', String(visibleCount))
            .replace('{total}', String(totalCount));
    }

    window.d2f.storage.saveState();
}

function toggleSearchToolbar(show?: boolean): void {
    const toolbar = document.getElementById('search-toolbar');
    const toggleBtn = document.getElementById('search-toggle-btn');
    const rawInput = document.getElementById('search-input');
    const input = rawInput instanceof HTMLInputElement ? rawInput : null;
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

function resetSearch(): void {
    preSearchCollapsedState = null;
    lastMatchedSectionIds.clear();

    const rawSearchInput = document.getElementById('search-input');
    const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
    if (searchInput) {
        searchInput.value = '';
    }

    const toolbar = document.getElementById('search-toolbar');
    if (toolbar && !toolbar.classList.contains('hidden')) {
        toggleSearchToolbar(false);
    } else {
        performSearchAndFilter();
    }
}

if (typeof window !== 'undefined') {
    document.addEventListener('DOMContentLoaded', () => {
        window.d2f.core.registerResetHandler(resetSearch);

        const searchToggleBtn = document.getElementById('search-toggle-btn');
        if (searchToggleBtn) {
            searchToggleBtn.addEventListener('click', () => toggleSearchToolbar());
        }

        const rawSearchInput = document.getElementById('search-input');
        const searchInput = rawSearchInput instanceof HTMLInputElement ? rawSearchInput : null;
        if (searchInput) {
            searchInput.addEventListener('input', () => performSearchAndFilter());
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
    });

    document.addEventListener('keydown', (e: KeyboardEvent) => {
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
}